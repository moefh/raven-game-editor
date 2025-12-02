mod asset_problem;
mod tileset;
mod map_data;
mod sprite;
mod mod_data;

use std::collections::BTreeMap;
use std::sync::LazyLock;

static TIMESTAMP_FORMAT: LazyLock<Vec<time::format_description::BorrowedFormatItem<'_>>> = LazyLock::new(
    || time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap());

use crate::data_asset::{DataAssetStore, DataAssetId};

pub use asset_problem::AssetProblem;

pub struct CheckResult {
    pub timestamp: String,
    pub asset_problems: BTreeMap<DataAssetId, Vec<AssetProblem>>,
}

impl CheckResult {
    pub fn check_project(store: &DataAssetStore) -> Self {
        let timestamp = if let Ok(now) = time::OffsetDateTime::now_local() && let Ok(timestamp) = now.format(&TIMESTAMP_FORMAT) {
            timestamp
        } else {
            "<unknown time>".to_owned()
        };

        let mut asset_problems = BTreeMap::new();
        for tileset in store.assets.tilesets.iter() {
            asset_problems.insert(tileset.asset.id, tileset::check_tileset(tileset, &store.assets));
        }
        for map_data in store.assets.maps.iter() {
            asset_problems.insert(map_data.asset.id, map_data::check_map(map_data, &store.assets));
        }
        for sprite in store.assets.sprites.iter() {
            asset_problems.insert(sprite.asset.id, sprite::check_sprite(sprite, &store.assets));
        }
        for mod_data in store.assets.mods.iter() {
            asset_problems.insert(mod_data.asset.id, mod_data::check_mod(mod_data, &store.assets));
        }

        CheckResult {
            timestamp,
            asset_problems,
        }
    }

    pub fn num_assets_checked(&self) -> usize {
        self.asset_problems.len()
    }

    pub fn num_assets_with_problems(&self) -> usize {
        self.asset_problems.values().filter(|problems| ! problems.is_empty()).count()
    }
}
