mod asset_problem;
mod tileset;
mod map_data;
mod sprite;
mod mod_data;

use std::collections::BTreeMap;
use std::sync::LazyLock;

static TIMESTAMP_FORMAT: LazyLock<Vec<time::format_description::BorrowedFormatItem<'_>>> = LazyLock::new(|| {
    time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap()
});

use crate::data_asset::{DataAssetStore, DataAssetId};

pub use asset_problem::{AssetProblem, MapLayer};

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
        tileset::check_tilesets(&mut asset_problems, store);
        map_data::check_maps(&mut asset_problems, store);
        sprite::check_sprites(&mut asset_problems, store);
        mod_data::check_mods(&mut asset_problems, store);

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
