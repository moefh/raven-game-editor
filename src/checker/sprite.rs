use crate::data_asset::{AssetCollection, Sprite};

use super::AssetProblem;

pub fn check_sprite(sprite: &Sprite, _assets: &AssetCollection) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if sprite.num_frames > 255 {
        problems.push(AssetProblem::SpriteTooBig { num_frames: sprite.num_frames });
    }

    problems
}
