use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, Sprite};

use super::{
    AssetError,
    AssetWarning,
};

fn check_sprite(sprite: &Sprite) -> (Vec<AssetError>, Vec<AssetWarning>) {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    if sprite.num_frames > 255 {
        errors.push(AssetError::SpriteTooBig { num_frames: sprite.num_frames });
    }

    (errors, warnings)
}

pub fn check_sprites(
    asset_errors: &mut BTreeMap<DataAssetId, Vec<AssetError>>,
    asset_warnings: &mut BTreeMap<DataAssetId, Vec<AssetWarning>>,
    store: &DataAssetStore
) {
    for sprite in store.assets.sprites.iter() {
        let (errors, warnings) = check_sprite(sprite);
        asset_errors.insert(sprite.asset.id, errors);
        asset_warnings.insert(sprite.asset.id, warnings);
    }
}
