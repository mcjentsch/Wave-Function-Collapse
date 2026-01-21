use bevy::ecs::component::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
}

impl Coord {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn neighbours(&self, max_row: usize, max_col: usize) -> [Option<(Direction, Coord)>; 4] {
        [
            if self.row > 0 {
                Some((Direction::Top, Coord::new(self.row - 1, self.col)))
            } else {
                None
            },
            if self.col > 0 {
                Some((Direction::Left, Coord::new(self.row, self.col - 1)))
            } else {
                None
            },
            if self.row + 1 < max_row {
                Some((Direction::Bottom, Coord::new(self.row + 1, self.col)))
            } else {
                None
            },
            if self.col + 1 < max_col {
                Some((Direction::Right, Coord::new(self.row, self.col + 1)))
            } else {
                None
            },
        ]
    }
}
