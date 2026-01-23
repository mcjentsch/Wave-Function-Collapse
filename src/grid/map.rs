use super::tile_data::TileData;
use super::{Coord, Tile};
use anyhow::Result;

#[derive(Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tile_data: TileData,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let tile_data = TileData::load("assets/tiledata.json")?;
        let domain = tile_data.tiles;
        let tiles = (0..height)
            .map(|_| (0..width).map(|_| Tile::new(None, domain)).collect())
            .collect();

        Ok(Self {
            width,
            height,
            tile_data,
            tiles,
        })
    }

    pub fn get_tile_mut(&mut self, coord: Coord) -> &mut Tile {
        &mut self.tiles[coord.row][coord.col]
    }
}
