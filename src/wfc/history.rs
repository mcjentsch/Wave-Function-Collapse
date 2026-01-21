use crate::grid::{Coord, TileType};

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
        removed: Vec<TileType>,
    },
    DomainReduction {
        coord: Coord,
        removed: Vec<TileType>,
        current_entropy: usize,
    },
}

pub enum VisualEvent {
    SetTile { tile_type: TileType, coord: Coord },
    UndoTile { coord: Coord },
}
