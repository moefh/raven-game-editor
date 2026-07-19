use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, Sprite};

use super::AssetProblem;

fn check_sprite(sprite: &Sprite) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if sprite.num_frames > 255 {
        problems.push(AssetProblem::SpriteTooBig { num_frames: sprite.num_frames });
    }

    problems
}

pub fn check_sprites(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for sprite in store.assets.sprites.iter() {
        asset_problems.insert(sprite.asset.id, check_sprite(sprite));
    }
}
