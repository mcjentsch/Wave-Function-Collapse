use super::tile_data::{Domain, TileType};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub tile_type: Option<TileType>,
    pub current_domain: Domain,
}

impl Tile {
    pub fn new(tile_type: Option<TileType>, current_domain: Domain) -> Self {
        Tile {
            tile_type,
            current_domain,
        }
    }

    pub fn get_current_domain_size(&self) -> usize {
        self.current_domain.entropy() as usize
    }

    pub fn reset_domain_to(&mut self, domain: Domain) {
        self.current_domain = domain;
    }

    pub fn update_constraints(&mut self, new_constraints: Domain) -> Option<Domain> {
        let removed: Domain = self.current_domain.difference(new_constraints);

        self.current_domain = self.current_domain.intersection(new_constraints);

        if removed.is_empty() {
            return None;
        }

        if self.current_domain.entropy() == 1 {
            self.tile_type = self.current_domain.as_single_tile();
        }

        Some(removed)
    }

    pub fn remove_contradiction_from_domain(&mut self, tile_type: TileType) {
        self.current_domain.remove_tile(tile_type);
    }

    pub fn collapse_self(&mut self) -> Result<(TileType, Domain)> {
        let mut removed = self.current_domain;

        let collapsed_tile = self
            .current_domain
            .collapse_domain()
            .ok_or_else(|| anyhow::anyhow!("Cannot collapse tile with empty current_domain"))?;

        removed = collapsed_tile.mask() ^ removed;

        self.tile_type = Some(collapsed_tile);

        Ok((collapsed_tile, removed))
    }
}
