use super::tile_data::TileType;
use anyhow::Result;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub tile_type: Option<TileType>,
    pub current_domain: HashSet<TileType>,
}

impl Tile {
    pub fn new(tile_type: Option<TileType>, current_domain: HashSet<TileType>) -> Self {
        Tile {
            tile_type,
            current_domain,
        }
    }

    pub fn get_current_domain_size(&self) -> usize {
        self.current_domain.len()
    }

    pub fn get_current_domain(&self) -> &HashSet<TileType> {
        &self.current_domain
    }

    pub fn reset_domain(&mut self, domain: &HashSet<TileType>) {
        self.current_domain.clone_from(domain);
    }

    pub fn update_constraints(
        &mut self,
        new_constraints: &HashSet<TileType>,
    ) -> Option<Vec<TileType>> {
        let removed: Vec<TileType> = self
            .current_domain
            .difference(new_constraints)
            .copied()
            .collect();

        self.current_domain
            .retain(|tile| new_constraints.contains(tile));

        if removed.is_empty() {
            return None;
        }

        if self.current_domain.len() == 1 {
            self.tile_type = self.current_domain.iter().copied().next();
        }

        Some(removed)
    }

    pub fn remove_contradiction_from_domain(&mut self, tile_type: &TileType) {
        self.current_domain.remove(tile_type);
    }

    pub fn collapse_self(&mut self) -> Result<(TileType, Vec<TileType>)> {
        let collapsed_tile = *self
            .current_domain
            .iter()
            .choose(&mut rand::rng())
            .ok_or_else(|| anyhow::anyhow!("Cannot collapse tile with empty current_domain"))?;

        let removed: Vec<TileType> = self
            .current_domain
            .iter()
            .filter(|&&t| t != collapsed_tile)
            .copied()
            .collect();

        self.tile_type = Some(collapsed_tile);
        self.current_domain = HashSet::from([collapsed_tile]);

        Ok((collapsed_tile, removed))
    }
}
