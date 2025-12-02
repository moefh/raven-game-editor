use crate::data_asset::{AssetCollection, Tileset};

use super::AssetProblem;

pub fn check_tileset(tileset: &Tileset, _assets: &AssetCollection) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if tileset.num_tiles > 255 {
        problems.push(AssetProblem::TilesetTooBig { num_tiles: tileset.num_tiles });
    }

    problems
}
