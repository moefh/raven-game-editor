use std::collections::{HashMap, BTreeMap};

use crate::image::{colors, ImageCollection};
use crate::data_asset::{DataAssetId, DataAssetStore, MapData, Tileset};

use super::{
    AssetError,
    AssetWarning,
    MapLayer,
};

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

fn get_bb_tile(map_data: &MapData, x: u32, y: u32) -> u8 {
    if ! map_data.has_bb || x >= map_data.width || y >= map_data.height { return MapData::NO_TILE; }
    map_data.bb_tiles[(y*map_data.width + x) as usize]
}

/// check for spots not covered by an opaque tile
fn check_map_transparency(map_data: &MapData, tileset_transp: &[bool], errors: &mut Vec<AssetError>) {
    // Build a map of all fg tile positions that may overlap a
    // transparent spot (i.e., bg and bb with no tile set).
    let mut fg_overlaps_transp = vec![false; (map_data.width * map_data.height) as usize];
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            if get_bg_tile(map_data, x, y) == MapData::NO_TILE && get_bb_tile(map_data, x, y) == MapData::NO_TILE {
                fg_overlaps_transp[(y*map_data.width + x) as usize] = true;
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
        errors.push(AssetError::MapTransparentTile {
            first_tile_x: first_bad_tile_x,
            first_tile_y: first_bad_tile_y,
            num_tiles: num_bad_tiles,
        });
    }
}

/// check for spots with double opaque tile cover (i.e., both bg and fg are opaque)
fn check_map_opaqueness(map_data: &MapData, tileset_transp: &[bool], errors: &mut Vec<AssetWarning>) {
    // Build a map of all fg tile positions that may overlap a
    // an opaque spot (i.e., a bg with opaque tile set).
    let mut fg_overlaps_opaque = vec![false; (map_data.width * map_data.height) as usize];
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            if get_bg_tile(map_data, x, y) != MapData::NO_TILE {
                fg_overlaps_opaque[(y*map_data.width + x) as usize] = true;
            }
        }
    }

    // For each fg tile position that can overlap an opaque spot, check if it contains an opaque tile.
    let mut num_bad_tiles = 0;
    let mut first_bad_tile_x = 0;
    let mut first_bad_tile_y = 0;
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            if ! fg_overlaps_opaque[(y*map_data.width + x) as usize] { continue; }
            let fg_tile = get_fg_tile(map_data, x, y);
            if fg_tile == MapData::NO_TILE || fg_tile as usize >= tileset_transp.len() {
                // no tile or invalid tile (will be caught by another checker)
                continue;
            }
            if ! tileset_transp[fg_tile as usize] {
                // check fails: fg is a fully opaque tile that overlaps a background with opaque tile
                if num_bad_tiles == 0 {
                    first_bad_tile_x = x;
                    first_bad_tile_y = y;
                }
                num_bad_tiles += 1;
            }
        }
    }

    if num_bad_tiles > 0 {
        errors.push(AssetWarning::MapOpaqueTile {
            first_tile_x: first_bad_tile_x,
            first_tile_y: first_bad_tile_y,
            num_tiles: num_bad_tiles,
        });
    }

}

fn check_map_tiles(map_data: &MapData, tileset: &Tileset, errors: &mut Vec<AssetError>) {
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_fg_tile(map_data, x, y);
            if tile != MapData::NO_TILE && tile as u32 >= tileset.num_tiles {
                errors.push(AssetError::MapInvalidTile {
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
                errors.push(AssetError::MapInvalidTile {
                    tile_x: x,
                    tile_y: y,
                    tile,
                    layer: MapLayer::Background,
                });
            }
        }
    }

    if map_data.has_bb {
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_bb_tile(map_data, x, y);
                if tile != MapData::NO_TILE && tile as u32 >= tileset.num_tiles {
                    errors.push(AssetError::MapInvalidTile {
                        tile_x: x,
                        tile_y: y,
                        tile,
                        layer: MapLayer::Background2,
                    });
                }
            }
        }
    }
}

pub fn check_maps(
    asset_errors: &mut BTreeMap<DataAssetId, Vec<AssetError>>,
    asset_warnings: &mut BTreeMap<DataAssetId, Vec<AssetWarning>>,
    store: &DataAssetStore
) {
    let mut tileset_transp_map = HashMap::new();
    for map_data in store.assets.maps.iter() {
        let mut map_errors = Vec::new();
        let mut map_warnings = Vec::new();

        if let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
            check_map_tiles(map_data, tileset, &mut map_errors);
            let tileset_transp = tileset_transp_map.entry(map_data.tileset_id).or_insert_with(|| build_tileset_transparency(tileset));
            check_map_transparency(map_data, tileset_transp, &mut map_errors);
            check_map_opaqueness(map_data, tileset_transp, &mut map_warnings);
        } else {
            map_errors.push(AssetError::MapTilesetInvalid { tileset_id: map_data.tileset_id });
        }
        asset_errors.insert(map_data.asset.id, map_errors);
        asset_warnings.insert(map_data.asset.id, map_warnings);
    }
}
