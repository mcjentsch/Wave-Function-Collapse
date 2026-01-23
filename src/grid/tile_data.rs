use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct TileDataRaw {
    pub tiles: Vec<TileType>,
    pub supports: HashMap<TileType, TileConstraintsRaw>,
}



#[derive(Deserialize, Debug)]
pub struct TileData {
    pub tiles: Domain,
    pub supports: HashMap<TileType, TileConstraints>,
}


impl TileData {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let raw_data: TileDataRaw = serde_json::from_reader(BufReader::new(file))?;
        
        let tiles = Domain::from_tiles(&raw_data.tiles);
        
        let supports = raw_data.supports
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            TileConstraints {
                                top: Domain::from_tiles(&v.top),
                                right: Domain::from_tiles(&v.right),
                                bottom: Domain::from_tiles(&v.bottom),
                                left: Domain::from_tiles(&v.left),
                            },
                        )
                    })
                    .collect();
        
        Ok(TileData { tiles, supports })
}
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileConstraintsRaw {
    pub top: Vec<TileType>,
    pub right: Vec<TileType>,
    pub bottom: Vec<TileType>,
    pub left: Vec<TileType>,
}


#[derive(Deserialize, Debug, Clone)]
pub struct TileConstraints {
    pub top: Domain,
    pub right: Domain,
    pub bottom: Domain,
    pub left: Domain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum TileType {
    // Water types
    DeepWater = 0,
    ShallowWater = 1,
    River = 2,

    // Land types
    Beach = 3,
    Grass = 4,
    Forest = 5,
    Mountain = 6,
    Snow = 7,
    Desert = 8,

    // Beach-Water transitions (edges)
    BeachWaterN = 9,
    BeachWaterE = 10,
    BeachWaterS = 11,
    BeachWaterW = 12,

    // Beach-Water transitions (corners)
    BeachWaterNe = 13,
    BeachWaterNw = 14,
    BeachWaterSe = 15,
    BeachWaterSw = 16,

    // Grass-Forest transitions
    GrassForestN = 17,
    GrassForestE = 18,
    GrassForestS = 19,
    GrassForestW = 20,

    // Mountain-Snow transitions
    MountainSnowN = 21,
    MountainSnowE = 22,
    MountainSnowS = 23,
    MountainSnowW = 24,
}

impl TileType {
    pub fn mask(self) -> Domain {
        Domain(1u32 << self as u8)
    }

    pub fn from_repr(num: u8) -> Option<Self> {
        match num {
            0 => Some(TileType::DeepWater),
            1 => Some(TileType::ShallowWater),
            2 => Some(TileType::River),
            3 => Some(TileType::Beach),
            4 => Some(TileType::Grass),
            5 => Some(TileType::Forest),
            6 => Some(TileType::Mountain),
            7 => Some(TileType::Snow),
            8 => Some(TileType::Desert),
            9 => Some(TileType::BeachWaterN),
            10 => Some(TileType::BeachWaterE),
            11 => Some(TileType::BeachWaterS),
            12 => Some(TileType::BeachWaterW),
            13 => Some(TileType::BeachWaterNe),
            14 => Some(TileType::BeachWaterNw),
            15 => Some(TileType::BeachWaterSe),
            16 => Some(TileType::BeachWaterSw),
            17 => Some(TileType::GrassForestN),
            18 => Some(TileType::GrassForestE),
            19 => Some(TileType::GrassForestS),
            20 => Some(TileType::GrassForestW),
            21 => Some(TileType::MountainSnowN),
            22 => Some(TileType::MountainSnowE),
            23 => Some(TileType::MountainSnowS),
            24 => Some(TileType::MountainSnowW),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Domain(pub u32);

impl Domain {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
    
    pub fn from_tiles(tiles: &[TileType]) -> Self{    
        tiles.iter().fold(Domain(0), |acc, tile| {
            acc | tile.mask()
        })
    }

    pub fn entropy(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn intersection(self, domain: Domain) -> Self {
        Self(self.0 & domain.0)
    }

    pub fn difference(self, domain: Domain) -> Self {
        Self(self.0 & !domain.0)
    }

    pub fn as_single_tile(self) -> Option<TileType> {
        if self.entropy() != 1 {
            return None;
        }

        let bit_position = self.0.trailing_zeros();

        if bit_position > 24 {
            return None;
        }

        TileType::from_repr(bit_position as u8)
    }

    pub fn remove_tile(&mut self, tile: TileType) {
        self.0 &= !tile.mask().0;
    }

    pub fn collapse_domain(&mut self) -> Option<TileType> {
        
        if self.entropy() == 0{
            return None;
        }
        
        let random_index = rand::random_range(0..self.entropy());

        for _ in 0..random_index {
            self.0 &= self.0 - 1
        }

        let index = self.0.trailing_zeros();
        self.0 = 1u32 << index;
        TileType::from_repr(index as u8)
        
    }

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn add_tiles(&mut self, tiles: Domain) {
        self.0 |= tiles.0;
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = TileType> {
        let mut mask = self.0;

        std::iter::from_fn(move || {
            if mask == 0 {
                return None;
            } 
            else {
                let index = mask.trailing_zeros();
                mask &= mask - 1;
                Some(Domain(1 << index))
            }
        })
        .filter_map(|domain| domain.as_single_tile())
    }
}

impl BitAnd for Domain {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Domain {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for Domain {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for Domain {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
