use super::tile_data::TileData;
use super::{Coord, Tile};
use anyhow::Result;
use std::sync::Arc;

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
        let domain = Arc::clone(&tile_data.tiles);
        let tiles = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| Tile::new(None, (*domain).clone()))
                    .collect()
            })
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

    pub fn print(&self) {
        println!("Tile Types:");
        for row in &self.tiles {
            for tile in row {
                let s = match tile.tile_type {
                    Some(t) => format!("{:?}", t),
                    None => "None".to_string(),
                };
                print!("{:>15} ", s);
            }
            println!();
        }

        println!("\nEntropy (domain size):");
        for row in &self.tiles {
            for tile in row {
                print!("{:>15} ", tile.current_domain.len());
            }
            println!();
        }

        println!("\nDomains:");
        for (row_idx, row) in self.tiles.iter().enumerate() {
            for (col_idx, tile) in row.iter().enumerate() {
                println!("({},{}): {:?}", row_idx, col_idx, tile.current_domain);
            }
        }

        println!("\nCoordinates:");
        for (row_idx, row) in self.tiles.iter().enumerate() {
            for col_idx in 0..row.len() {
                print!("{:>15} ", format!("({},{})", row_idx, col_idx));
            }
            println!();
        }
    }
}
