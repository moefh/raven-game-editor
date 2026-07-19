use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, Tileset};

use super::AssetProblem;

fn check_tileset(tileset: &Tileset) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if tileset.num_tiles > 255 {
        problems.push(AssetProblem::TilesetTooBig { num_tiles: tileset.num_tiles });
    }

    problems
}

pub fn check_tilesets(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for tileset in store.assets.tilesets.iter() {
        asset_problems.insert(tileset.asset.id, check_tileset(tileset));
    }
}
