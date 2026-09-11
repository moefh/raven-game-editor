mod asset_problem;
mod tileset;
mod map_data;
mod sprite;
mod pal_sprite;
mod mod_data;
mod room;
mod world;

use std::collections::BTreeMap;

pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 240;

use crate::platform::current_time_as_string;
use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
};

pub use asset_problem::{
    AssetError,
    AssetWarning,
    MapLayer,
};

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
    pub asset_errors: BTreeMap<DataAssetId, Vec<AssetError>>,
    pub asset_warnings: BTreeMap<DataAssetId, Vec<AssetWarning>>,
    pub merged_samples: Vec<MergedSample>,
    pub merged_samples_saved_size: usize,
    pub data_size: usize,
}

impl CheckResult {
    pub fn check_project(store: &DataAssetStore) -> Self {
        let timestamp = current_time_as_string();

        let mut asset_errors = BTreeMap::new();
        let mut asset_warnings = BTreeMap::new();
        tileset::check_tilesets(&mut asset_errors, &mut asset_warnings, store);
        map_data::check_maps(&mut asset_errors, &mut asset_warnings, store);
        sprite::check_sprites(&mut asset_errors, &mut asset_warnings, store);
        pal_sprite::check_pal_sprites(&mut asset_errors, &mut asset_warnings, store);
        mod_data::check_mods(&mut asset_errors, &mut asset_warnings, store);
        room::check_rooms(&mut asset_errors, &mut asset_warnings, store);
        world::check_worlds(&mut asset_errors, &mut asset_warnings, store);

        let merged_samples = mod_data::check_merged_samples(store);
        let merged_samples_saved_size = merged_samples.iter().fold(0, |sum, m| sum + m.saved_size);
        let data_size = store.assets.data_size() - merged_samples_saved_size;

        CheckResult {
            timestamp,
            asset_errors,
            asset_warnings,
            merged_samples,
            merged_samples_saved_size,
            data_size,
        }
    }

    pub fn num_assets_checked(&self) -> usize {
        self.asset_errors.len()
    }

    pub fn num_assets_with_errors(&self) -> usize {
        self.asset_errors.values().filter(|errors| ! errors.is_empty()).count()
    }

    pub fn num_assets_with_warnings(&self) -> usize {
        self.asset_warnings.values().filter(|warnings| ! warnings.is_empty()).count()
    }
}
