use std::collections::{HashMap, BTreeMap};

use crate::image::{colors, ImageCollection};
use crate::data_asset::{DataAssetId, DataAssetStore, MapData, Tileset};

use super::{AssetProblem, MapLayer};

pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 240;

/// Return a vector of bools indicating whether each tile is transparent or not
fn build_tileset_transparency(tileset: &Tileset) -> Vec<bool> {
    (0..tileset.num_tiles).map(|tile| {
        tileset.item_data(tile).contains(&colors::TRANSPARENT)
    }).collect()
}

fn get_fg_tile(map_data: &MapData, x: u32, y: u32) -> u8 {
    if x >= map_data.width || y >= map_data.height { return MapData::NO_TILE; }
    map_data.fg_tiles[(y*map_data.width + x) as usize]
}

fn get_bg_tile(map_data: &MapData, x: u32, y: u32) -> u8 {
    if x >= map_data.width || y >= map_data.height { return MapData::NO_TILE; }
    map_data.bg_tiles[(y*map_data.width + x) as usize]
}

fn get_para_tile(map_data: &MapData, x: u32, y: u32) -> u8 {
    if x >= map_data.para_width || y >= map_data.para_height { return MapData::NO_TILE; }
    map_data.para_tiles[(y*map_data.para_width + x) as usize]
}

fn check_map_transparency(map_data: &MapData, tileset_transp: &[bool], problems: &mut Vec<AssetProblem>) {
    if map_data.para_width > map_data.width || map_data.para_height > map_data.height {
        // invalid size; this will be caught by another checker
        return;
    }

    // Build a map of all fg tile positions that may overlap a
    // transparent spot (i.e., a bg or parallax with no tile set).
    let mut fg_overlaps_transp = vec![false; (map_data.width * map_data.height) as usize];
    if map_data.para_width == 0 || map_data.para_height == 0 {
        // if there's no parallax layer, just check the background
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                if get_bg_tile(map_data, x, y) == MapData::NO_TILE {
                    fg_overlaps_transp[(y*map_data.width + x) as usize] = true;
                }
            }
        }
    } else {
        // Note that bg or parallax with a tile set is never
        // transparent, because in these layers a transparent pixel is
        // drawn as green.
        let pw = map_data.width - map_data.para_width + 1;
        let ph = map_data.height - map_data.para_height + 1;
        for y in 0..map_data.para_height {
            for x in 0..map_data.para_width {
                if get_para_tile(map_data, x, y) != MapData::NO_TILE { continue; }
                for py in 0..ph {
                    for px in 0..pw {
                        if get_bg_tile(map_data, x+px, y+py) == MapData::NO_TILE {
                            fg_overlaps_transp[((y+py)*map_data.width + x+px) as usize] = true;
                        }
                    }
                }
            }
        }
    }

    // For each fg tile position that can overlap a transparent spot, check if it contains a transparent tile.
    let mut num_bad_tiles = 0;
    let mut first_bad_tile_x = 0;
    let mut first_bad_tile_y = 0;
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            if ! fg_overlaps_transp[(y*map_data.width + x) as usize] { continue; }
            let fg_tile = get_fg_tile(map_data, x, y);
            if fg_tile != MapData::NO_TILE && fg_tile as usize >= tileset_transp.len() {
                // invalid fg tile; this will be caught by another checker
                continue;
            }
            if fg_tile == MapData::NO_TILE || tileset_transp[fg_tile as usize] {
                // check fails: fg has <no tile or tile with transparent pixel> that <can overlap a background with no tile>
                if num_bad_tiles == 0 {
                    first_bad_tile_x = x;
                    first_bad_tile_y = y;
                }
                num_bad_tiles += 1;
            }
        }
    }

    if num_bad_tiles > 0 {
        problems.push(AssetProblem::MapTransparentTile {
            first_tile_x: first_bad_tile_x,
            first_tile_y: first_bad_tile_y,
            num_tiles: num_bad_tiles,
        });
    }
}

fn check_map_tiles(map_data: &MapData, tileset: &Tileset, problems: &mut Vec<AssetProblem>) {
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_fg_tile(map_data, x, y);
            if tile != MapData::NO_TILE && tile as u32 >= tileset.num_tiles {
                problems.push(AssetProblem::MapInvalidTile {
                    tile_x: x,
                    tile_y: y,
                    tile,
                    layer: MapLayer::Foreground,
                });
            }
        }
    }

    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_bg_tile(map_data, x, y);
            if tile != MapData::NO_TILE && tile as u32 >= tileset.num_tiles {
                problems.push(AssetProblem::MapInvalidTile {
                    tile_x: x,
                    tile_y: y,
                    tile,
                    layer: MapLayer::Background,
                });
            }
        }
    }

    for y in 0..map_data.para_height {
        for x in 0..map_data.para_width {
            let tile = get_para_tile(map_data, x, y);
            if tile != MapData::NO_TILE && tile as u32 >= tileset.num_tiles {
                problems.push(AssetProblem::MapInvalidTile {
                    tile_x: x,
                    tile_y: y,
                    tile,
                    layer: MapLayer::Parallax,
                });
            }
        }
    }
}

fn check_map_size(map_data: &MapData, problems: &mut Vec<AssetProblem>) {
    if (map_data.width * Tileset::TILE_SIZE) < SCREEN_WIDTH || (map_data.height * Tileset::TILE_SIZE) < SCREEN_HEIGHT {
        problems.push(AssetProblem::MapTooSmall {
            width: map_data.width,
            height: map_data.height,
        });
    }

    if map_data.para_width != 0 && map_data.height != 0 &&
        ((map_data.para_width * Tileset::TILE_SIZE) < SCREEN_WIDTH ||
         (map_data.para_height * Tileset::TILE_SIZE) < SCREEN_HEIGHT) {
            problems.push(AssetProblem::MapParallaxTooSmall {
                para_width: map_data.para_width,
                para_height: map_data.para_height,
            });
        }

    if map_data.para_width > map_data.width || map_data.para_height > map_data.height {
        problems.push(AssetProblem::MapParallaxTooBig {
            width: map_data.width,
            height: map_data.height,
            para_width: map_data.para_width,
            para_height: map_data.para_height,
        });
    }
}

pub fn check_maps(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    let mut tileset_transp_map = HashMap::new();
    for map_data in store.assets.maps.iter() {
        let mut map_problems = Vec::new();

        if let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
            check_map_size(map_data, &mut map_problems);
            check_map_tiles(map_data, tileset, &mut map_problems);
            let tileset_transp = tileset_transp_map.entry(map_data.tileset_id).or_insert_with(|| build_tileset_transparency(tileset));
            check_map_transparency(map_data, tileset_transp, &mut map_problems);
        } else {
            map_problems.push(AssetProblem::MapTilesetInvalid { tileset_id: map_data.tileset_id });
        }
        asset_problems.insert(map_data.asset.id, map_problems);
    }
}
