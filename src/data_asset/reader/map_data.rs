use std::io::Result;

use super::{
    error,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    MapData,
    MapTileChange,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("w"), ValueDef::U16),
        (String::from("h"), ValueDef::U16),
        (String::from("has_bb"), ValueDef::U8),
        (String::from("num_tile_changes"), ValueDef::U16),
        (String::from("tileset"), ValueDef::AssetRef),
        (String::from("tile_anim"), ValueDef::AssetRef),
        (String::from("tiles"), ValueDef::ArrayRef),
        (String::from("tile_changes"), ValueDef::ArrayRef),   // MAP_TILE_CHANGE
    ])
}

pub fn get_global_struct_defs() -> Vec<(String, ValueDefStruct)> {
    vec![
        (String::from("MAP_TILE_CHANGE"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("tile_bitmap"), ValueDef::ArrayRef),
            (String::from("tiles"), ValueDef::ArrayRef),
        ])),
    ]
}

fn conv_tiles(
    width: u32,
    height: u32,
    tile_bitmap: &[u32],
    tiles: &[u8],
    tile_pos: super::TokenPosition
) -> Result<Vec<Option<u8>>> {
    let width = width as usize;
    let height = height as usize;
    let mut ret_tiles = vec![None; width * height];
    let mut tile_bitmap_index = 0;
    let mut tile_index = 0;
    for y in 0..height {
        for x in 0..width {
            if (tile_bitmap[tile_bitmap_index/32] & (1 << (tile_bitmap_index%32))) != 0 {
                let tile = tiles.get(tile_index).copied();
                if tile.is_some() {
                    ret_tiles[y * width + x] = tile;
                } else {
                    return error(
                        format!(
                            "map tile change data is too small: {} <= {}",
                            tiles.len(),
                            tile_index
                        ),
                        tile_pos
                    );
                }
                tile_index += 1;
            }
            tile_bitmap_index += 1;
        }
    }
    Ok(ret_tiles)
}

fn conv_tile_change(change: &ValueStruct, name: String, project_data: &ProjectData) -> Result<MapTileChange> {
    let width = change.get_u16("width")?;
    let height = change.get_u16("height")?;
    let tile_bitmap_array = change.get_array_ref("tile_bitmap")?;
    let tiles_array = change.get_array_ref("tiles")?;

    let width = width as u32;
    let height = height as u32;

    let tile_bitmap = tile_bitmap_array.get_u32_array(project_data)?;
    let tiles_data = tiles_array.get_u8_array(project_data)?;
    let tiles = conv_tiles(width, height, tile_bitmap, tiles_data, tiles_array.pos)?;

    Ok(MapTileChange {
        name,
        width,
        height,
        tiles,
    })
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<MapData> {
    let width = asset_struct.get_u16("w")?;
    let height = asset_struct.get_u16("h")?;
    let has_bb = asset_struct.get_u8("has_bb")? != 0;
    let tileset_ref = asset_struct.get_asset_ref("tileset")?;
    let tile_anim_ref = asset_struct.get_asset_ref("tile_anim")?;
    let array = asset_struct.get_array_ref("tiles")?;
    let num_tile_changes = asset_struct.get_u16("num_tile_changes")?;
    let tile_changes_array = asset_struct.get_array_ref("tile_changes")?;

    let name = project_data.extract_asset_name("map_tiles_", array)?;
    let tiles = array.get_u8_array(project_data)?;

    let width = width as u32;
    let height = height as u32;

    let want_len = if has_bb { 4 } else { 3 } * width * height;
    if tiles.len() as u32 != want_len {
        error(
            format!(
                "unexpected tiles data length: got {}, expected {} = {}*{}*{}",
                tiles.len(), want_len, if has_bb { 4 } else { 3 }, width, height
            ),
            array.pos
        )?;
    }

    let fg_size = (width * height) as usize;
    let fg_tiles = Vec::from(&tiles[0..fg_size]);
    let bg_tiles = Vec::from(&tiles[fg_size..2*fg_size]);
    let fx_tiles = Vec::from(&tiles[2*fg_size..3*fg_size]);
    let bb_tiles = Vec::from(&tiles[3*fg_size..]);

    let tile_changes = match tile_changes_array.get_struct_array_or_null(project_data)? {
        Some(tile_changes_data) => {
            let tile_changes = tile_changes_data.values.iter().enumerate().map(|(index, change)| {
                let name = project_data
                    .get_asset_data_name(index, "MAP", name, "TILE_CHANGE")
                    .unwrap_or_else(|| format!("tile_change_{}", index));
                conv_tile_change(change, name, project_data)
            }).collect::<Result<Vec<_>>>()?;

            if num_tile_changes as usize != tile_changes.len() {
                return error(
                    format!("invalid number of map tile changes: expected {}, got {}", num_tile_changes, tile_changes.len()),
                    tile_changes_array.pos
                );
            }

            tile_changes
        }
        None => {
            if num_tile_changes != 0 {
                return error(
                    "map tile changes can't be NULL with nonzero num tile changes",
                    tile_changes_array.pos
                );
            }
            Vec::new()
        }
    };

    Ok(MapData {
        asset: DataAsset::new(DataAssetType::MapData, asset_id, DataAsset::identifier_to_name(name)),
        tileset_id: tileset_ref.get_asset_id(project_data)?,
        tile_anim_id: tile_anim_ref.get_asset_id_or_none(project_data)?,
        width,
        height,
        has_bb,
        fg_tiles,
        bg_tiles,
        fx_tiles,
        bb_tiles,
        tile_changes,
    })
}
