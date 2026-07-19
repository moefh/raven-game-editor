mod asset_problem;
mod tileset;
mod map_data;
mod sprite;
mod pal_sprite;
mod mod_data;
mod room;
mod world;

use std::collections::BTreeMap;
use std::sync::LazyLock;

pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 240;

static TIMESTAMP_FORMAT: LazyLock<time::format_description::FormatDescriptionV3> = LazyLock::new(|| {
    time::format_description::parse_borrowed::<3>("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap()
});

use crate::data_asset::{DataAssetStore, DataAssetId};

pub use asset_problem::{AssetProblem, MapLayer};

pub struct MergedSample {
    pub saved_size: usize,

    // MOD sample that was merged:
    pub merged_mod_id: DataAssetId,
    pub merged_sample_index: usize,

    // MOD sample it was merged to (whose data will be used):
    pub data_mod_id: DataAssetId,
    pub data_sample_index: usize,
}

pub struct CheckResult {
    pub timestamp: String,
    pub asset_problems: BTreeMap<DataAssetId, Vec<AssetProblem>>,
    pub merged_samples: Vec<MergedSample>,
    pub merged_samples_saved_size: usize,
    pub data_size: usize,
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
        pal_sprite::check_pal_sprites(&mut asset_problems, store);
        mod_data::check_mods(&mut asset_problems, store);
        room::check_rooms(&mut asset_problems, store);
        world::check_worlds(&mut asset_problems, store);

        let merged_samples = mod_data::check_merged_samples(store);
        let merged_samples_saved_size = merged_samples.iter().fold(0, |sum, m| sum + m.saved_size);
        let data_size = store.assets.data_size() - merged_samples_saved_size;

        CheckResult {
            timestamp,
            asset_problems,
            merged_samples,
            merged_samples_saved_size,
            data_size,
        }
    }

    pub fn num_assets_checked(&self) -> usize {
        self.asset_problems.len()
    }

    pub fn num_assets_with_problems(&self) -> usize {
        self.asset_problems.values().filter(|problems| ! problems.is_empty()).count()
    }
}
