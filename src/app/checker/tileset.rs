use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, Tileset};

use super::{
    AssetError,
    AssetWarning,
};

fn check_tileset(tileset: &Tileset) -> (Vec<AssetError>, Vec<AssetWarning>) {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    if tileset.num_tiles > 255 {
        errors.push(AssetError::TilesetTooBig { num_tiles: tileset.num_tiles });
    }

    (errors, warnings)
}

pub fn check_tilesets(
    asset_errors: &mut BTreeMap<DataAssetId, Vec<AssetError>>,
    asset_warnings: &mut BTreeMap<DataAssetId, Vec<AssetWarning>>,
    store: &DataAssetStore
) {
    for tileset in store.assets.tilesets.iter() {
        let (errors, warnings) = check_tileset(tileset);
        asset_errors.insert(tileset.asset.id, errors);
        asset_warnings.insert(tileset.asset.id, warnings);
    }
}
