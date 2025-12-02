use crate::data_asset::{AssetCollection, MapData, Tileset};

use super::AssetProblem;

pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 240;

pub fn check_map(map_data: &MapData, _assets: &AssetCollection) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if (map_data.width * Tileset::TILE_SIZE) < SCREEN_WIDTH || (map_data.height * Tileset::TILE_SIZE) < SCREEN_HEIGHT {
        problems.push(AssetProblem::MapTooSmall { width: map_data.width, height: map_data.height });
    } else if (map_data.bg_width * Tileset::TILE_SIZE) < SCREEN_WIDTH || (map_data.bg_height * Tileset::TILE_SIZE) < SCREEN_HEIGHT {
        problems.push(AssetProblem::MapBackgroundTooSmall { bg_width: map_data.bg_width, bg_height: map_data.bg_height });
    }

    if map_data.bg_width > map_data.width || map_data.bg_height > map_data.height {
        problems.push(AssetProblem::MapBackgroundTooBig {
            width: map_data.width,
            height: map_data.height,
            bg_width: map_data.bg_width,
            bg_height: map_data.bg_height,
        });
    }

    problems
}
