use crate::grid::{Coord, Domain, TileType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollapseKind {
    Explicit,
    Implicit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Collapse {
        kind: CollapseKind,
        tile_type: TileType,
        coord: Coord,
        removed: Domain,
    },
    DomainReduction {
        coord: Coord,
        removed: Domain,
        current_entropy: usize,
    },
}

pub enum VisualEvent {
    SetTile { tile_type: TileType, coord: Coord },
    UndoTile { coord: Coord },
}
