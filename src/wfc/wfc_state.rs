use super::history::{Action, CollapseKind, VisualEvent};
use crate::bucket_queue::BucketQueue;
use crate::grid::TileType;
use crate::grid::{Coord, Direction, Domain, Map};
use anyhow::{Result, anyhow};
use std::collections::{VecDeque};
use std::fmt;

#[derive(Debug)]
pub enum Contradiction {
    EmptyDomain { tile_type: TileType, coord: Coord },
    ExhaustedPaths { tile_type: TileType, coord: Coord },
}

impl fmt::Display for Contradiction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Contradiction::EmptyDomain { tile_type, coord } => {
                write!(
                    f,
                    "Contradiction at tile ({}, {}) for type {:?} - domain became empty during propagation",
                    coord.row, coord.col, tile_type
                )
            }
            Contradiction::ExhaustedPaths { tile_type, coord } => {
                write!(
                    f,
                    "No valid tile types remaining at ({}, {}) after removing {:?}",
                    coord.row, coord.col, tile_type
                )
            }
        }
    }
}

impl std::error::Error for Contradiction {}

pub struct WFCState {
    map: Map,
    least_entropy: BucketQueue,
    timeline: VecDeque<VisualEvent>,
    history: Vec<Action>,
}

impl Iterator for WFCState {
    type Item = VisualEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.timeline.pop_front().or_else(|| {
            self.least_entropy.peek_min()?;
            match self.collapse() {
                Ok((tile_type, coord)) => Some(VisualEvent::SetTile { tile_type, coord }),
                Err(e) => {
                    eprintln!("WFC Error: {}", e);
                    None
                }
            }
        })
    }
}

impl WFCState {
    pub fn new(map: Map) -> Self {
        let least_entropy = WFCState::set_initial_entropy(&map);

        WFCState {
            map,
            least_entropy,
            timeline: VecDeque::new(),
            history: Vec::new(),
        }
    }

    fn set_initial_entropy(map: &Map) -> BucketQueue {
        let mut queue = BucketQueue::new(map.tile_data.tiles.entropy() as usize);

        for (row_idx, row) in map.tiles.iter().enumerate() {
            for (col_idx, tile) in row.iter().enumerate() {
                let coord = Coord::new(row_idx, col_idx);
                let entropy = tile.get_current_domain_size();
                if let Err(insert) = queue.insert(coord, entropy) {
                    panic!("Failed to Insert: {:?}", insert);
                }
            }
        }

        queue
    }

    fn find_least_entropy(&mut self) -> Option<Coord> {
        self.least_entropy
            .extract_min()
            .map(|(coord, _entropy)| coord)
    }

    fn find_last_collapse(&self) -> Option<usize> {
        self.history.iter().rposition(|action| {
            matches!(
                action,
                Action::Collapse {
                    kind: CollapseKind::Explicit,
                    ..
                }
            )
        })
    }

    fn backtrack(&mut self, to: usize) -> anyhow::Result<()> {
        while self.history.len() > to {
            match self.history.pop() {
                Some(Action::Collapse { coord, removed, .. }) => {
                    self.undo_collapse(coord, removed)?
                }
                Some(Action::DomainReduction { coord, removed, .. }) => {
                    self.undo_domain_reduction(coord, removed)?
                }
                _ => continue,
            }
        }

        // Now undo the target collapse and remove the failed tile type
        if let Some(Action::Collapse {
            tile_type,
            coord,
            removed,
            ..
        }) = self.history.pop()
        {
            let tile = self.map.get_tile_mut(coord);
            tile.tile_type = None;
            tile.current_domain.add_tiles(removed);
            tile.remove_contradiction_from_domain(tile_type);

            if tile.get_current_domain_size() == 0 {
                return Err(Contradiction::ExhaustedPaths { tile_type, coord }.into());
            }

            let entropy = tile.get_current_domain_size();
            self.least_entropy.insert(coord, entropy)?;
            self.timeline.push_back(VisualEvent::UndoTile { coord });
        }
        Ok(())
    }

    fn undo_collapse(&mut self, coord: Coord, removed: Domain) -> anyhow::Result<()> {
        let tile = self.map.get_tile_mut(coord);
        tile.tile_type = None;
        tile.current_domain.add_tiles(removed);

        let entropy = tile.get_current_domain_size();
        self.least_entropy.insert(coord, entropy)?;
        self.timeline.push_back(VisualEvent::UndoTile { coord });
        Ok(())
    }

    fn undo_domain_reduction(&mut self, coord: Coord, removed: Domain) -> anyhow::Result<()> {
        let tile = self.map.get_tile_mut(coord);
        tile.current_domain.add_tiles(removed);
        let entropy = tile.get_current_domain_size();
        self.least_entropy.update_entropy(coord, entropy)?;
        Ok(())
    }

    fn collapse(&mut self) -> anyhow::Result<(TileType, Coord)> {
        loop {
            let chosen_cell = self
                .find_least_entropy()
                .ok_or_else(|| anyhow::anyhow!("No cells left to collapse"))?;

            let (chosen_tile_type, removed) = self.map.get_tile_mut(chosen_cell).collapse_self()?;

            self.timeline.push_back(VisualEvent::SetTile {
                tile_type: chosen_tile_type,
                coord: chosen_cell,
            });

            self.history.push(Action::Collapse {
                kind: CollapseKind::Explicit,
                tile_type: chosen_tile_type,
                coord: chosen_cell,
                removed,
            });

            let mut stack: Vec<Coord> = Vec::new();
            stack.push(chosen_cell);

            match self.propagate(chosen_cell, chosen_tile_type, &mut stack) {
                Ok(()) => return Ok((chosen_tile_type, chosen_cell)),
                Err(_) => {
                    let mut to = self
                        .find_last_collapse()
                        .map(|index| index + 1) // make it in terms of length
                        .ok_or_else(|| anyhow!("No Tile Found Go Back To"))?;

                    loop {
                        match self.backtrack(to) {
                            Ok(()) => break,
                            Err(e) => {
                                if let Some(Contradiction::ExhaustedPaths { .. }) =
                                    e.downcast_ref::<Contradiction>()
                                {
                                    to = self
                                        .find_last_collapse()
                                        .ok_or_else(|| anyhow!("No Tile Found Go Back To"))?;

                                    continue;
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn propagate(
        &mut self,
        chosen_cell: Coord,
        chosen_tile_type: TileType,
        changed_cells: &mut Vec<Coord>,
    ) -> Result<()> {
        while let Some(changed_cell) = changed_cells.pop() {
            let neighbours = changed_cell.neighbours(self.map.width, self.map.height);

            for (direction, coord) in neighbours.into_iter().flatten() {
                let current_tile_types =
                    self.map.tiles[changed_cell.row][changed_cell.col].current_domain;

                let mut all_supported_tile_types = Domain::empty();

                for current_tile_type in current_tile_types.iter_tiles() {
                    let tile_constraints = self
                        .map
                        .tile_data
                        .supports
                        .get(&current_tile_type)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Missing constraint data for tile type {:?}",
                                current_tile_type
                            )
                        })?;

                    let new_constraints = match direction {
                        Direction::Top => tile_constraints.top,
                        Direction::Bottom => tile_constraints.bottom,
                        Direction::Right => tile_constraints.right,
                        Direction::Left => tile_constraints.left,
                    };

                    all_supported_tile_types.add_tiles(new_constraints);
                }

                let current_tile = self.map.get_tile_mut(coord);

                // Skip tiles that are already collapsed
                if current_tile.tile_type.is_some() {
                    continue;
                }

                let entropy_before_update = current_tile.get_current_domain_size();

                let removed_tiles = current_tile.update_constraints(all_supported_tile_types);

                if let Some(removed) = removed_tiles {
                    let entropy_after_update = current_tile.get_current_domain_size();

                    changed_cells.push(coord);

                    if entropy_after_update == 0 {
                        self.history.push(Action::DomainReduction {
                            coord,
                            removed,
                            current_entropy: entropy_before_update,
                        });
                        return Err(Contradiction::EmptyDomain {
                            tile_type: chosen_tile_type,
                            coord: chosen_cell,
                        }
                        .into());
                    }

                    self.least_entropy
                        .update_entropy(coord, entropy_after_update)?;

                    self.history.push(Action::DomainReduction {
                        coord,
                        removed: removed.clone(),
                        current_entropy: entropy_after_update,
                    });

                    if entropy_after_update == 1 {
                        let tile_type = current_tile.tile_type.ok_or_else(|| {
                            anyhow::anyhow!(
                                "Cell {:?} collapsed but tile_type is None",
                                changed_cell
                            )
                        })?;

                        self.least_entropy.remove(coord)?;

                        self.timeline
                            .push_back(VisualEvent::SetTile { tile_type, coord });

                        self.history.push(Action::Collapse {
                            kind: CollapseKind::Implicit,
                            tile_type,
                            coord,
                            removed,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
