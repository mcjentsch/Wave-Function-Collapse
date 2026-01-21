use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct TileData {
    pub tiles: Arc<HashSet<TileType>>,
    pub supports: HashMap<TileType, TileConstraints>,
}

impl TileData {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(BufReader::new(file))?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileConstraints {
    pub top: HashSet<TileType>,
    pub right: HashSet<TileType>,
    pub bottom: HashSet<TileType>,
    pub left: HashSet<TileType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    // Water types
    DeepWater,
    ShallowWater,
    River,

    // Land types
    Beach,
    Grass,
    Forest,
    Mountain,
    Snow,
    Desert,

    // Beach-Water transitions (edges)
    BeachWaterN,
    BeachWaterE,
    BeachWaterS,
    BeachWaterW,

    // Beach-Water transitions (corners)
    BeachWaterNe,
    BeachWaterNw,
    BeachWaterSe,
    BeachWaterSw,

    // Grass-Forest transitions
    GrassForestN,
    GrassForestE,
    GrassForestS,
    GrassForestW,

    // Mountain-Snow transitions
    MountainSnowN,
    MountainSnowE,
    MountainSnowS,
    MountainSnowW,
}
