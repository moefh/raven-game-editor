use std::io::{Result, Error};
use std::collections::HashMap;

use super::{
    ProjectDataWriter,
    IdentStore,
};
use super::super::{
    DataAssetId,
    DataAssetType,
    MapData,
    MapTileChange,
};

fn write_map_tile_layer(writer: &ProjectDataWriter, tiles: &[u8]) {
    for (index, tile) in tiles.iter().enumerate() {
        if index.is_multiple_of(16) { writer.write("\n  "); }
        writer.write(format!("{:#04x},", tile))
    }
}

fn write_map_tiles(writer: &ProjectDataWriter, map_data: &MapData, name_id: &str) {
    writer.write(format!("static const uint8_t {}_map_tiles_{}[] = {{", writer.ident.prefix_lower, name_id));
    writer.write("\n  // foreground");
    write_map_tile_layer(writer, &map_data.fg_tiles);
    writer.write("\n  // background");
    write_map_tile_layer(writer, &map_data.bg_tiles);
    writer.write("\n  // effects");
    write_map_tile_layer(writer, &map_data.fx_tiles);
    if map_data.has_bb {
        writer.write("\n  // back-background");
        write_map_tile_layer(writer, &map_data.bb_tiles);
    }
    writer.write("\n};\n");
    writer.write("\n");
}

fn write_map_tile_change_tile_bitmap(
    writer: &ProjectDataWriter,
    change: &MapTileChange,
    change_index: usize,
    map_name_id: &str
) -> Result<()> {
    writer.write(format!(
        "static const uint32_t {}_map_{}_tile_change_{}_tile_bitmap[] = {{",
        writer.ident.prefix_lower,
        map_name_id,
        change_index
    ));
    let mut bits = 0;
    let mut num_bits = 0;
    let mut num_chunks = 0;
    for y in 0..change.height {
        for x in 0..change.width {
            if change.tiles[(y * change.width + x) as usize].is_some() {
                bits |= 1 << num_bits;
            }
            num_bits += 1;
            if num_bits == 32 {
                if num_chunks % 8 == 0 { writer.write("\n    "); }
                writer.write(format!("{:#010x},", bits));
                bits = 0;
                num_bits = 0;
                num_chunks += 1;
            }
        }
    }
    if num_bits != 0 {
        if num_chunks % 8 == 0 { writer.write("\n    "); }
        writer.write(format!("{:#010x},", bits));
    }

    writer.write("\n};\n\n");
    Ok(())
}

fn write_map_tile_change_tiles(
    writer: &ProjectDataWriter,
    change: &MapTileChange,
    change_index: usize,
    map_name_id: &str
) -> Result<()> {
    writer.write(format!(
        "static const uint8_t {}_map_{}_tile_change_{}_tiles[] = {{",
        writer.ident.prefix_lower,
        map_name_id,
        change_index
    ));

    let mut num_tiles_written = 0;
    for y in 0..change.height {
        for x in 0..change.width {
            if let Some(tile) = change.tiles[(y * change.width + x) as usize] {
                if num_tiles_written % 16 == 0 { writer.write("\n    "); }
                writer.write(format!("{:#04x},", tile));
                num_tiles_written += 1;
            }
        }
    }
    writer.write("\n};\n\n");
    Ok(())
}

fn write_map_tile_changes(writer: &ProjectDataWriter, map_data: &MapData, name_id: &str) -> Result<()> {
    if map_data.tile_changes.is_empty() { return Ok(()); }

    for (change_index, change) in map_data.tile_changes.iter().enumerate() {
        write_map_tile_change_tile_bitmap(writer, change, change_index, name_id)?;
        write_map_tile_change_tiles(writer, change, change_index, name_id)?;
    }

    writer.write(format!("static const struct {}_MAP_TILE_CHANGE {}_map_tile_changes_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for (change_index, change) in map_data.tile_changes.iter().enumerate() {
        writer.write("  {\n");
        writer.write(format!("    {}, {},\n", change.width, change.height));
        writer.write(format!("    {}_map_{}_tile_change_{}_tile_bitmap,\n", writer.ident.prefix_lower, name_id, change_index));
        writer.write(format!("    {}_map_{}_tile_change_{}_tiles,\n", writer.ident.prefix_lower, name_id, change_index));
        writer.write("  },\n");
    }
    writer.write("};\n");
    writer.write("\n");
    Ok(())
}

pub fn write_maps(writer: &ProjectDataWriter, map_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === MAPS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in map_ids.iter() {
        if let Some(map_data) = writer.store.assets.maps.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::MapData, *id)?;
            write_map_tiles(writer, map_data, name_id);
            write_map_tile_changes(writer, map_data, name_id)?;
        }
    }

    writer.log(format!("-> writing {} maps", writer.store.asset_ids.maps.len()));
    writer.write(format!("const struct {}_MAP {}_maps[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in map_ids.iter() {
        if let Some(map_data) = writer.store.assets.maps.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::MapData, *id)?;
            writer.write("  {\n");

            // size
            writer.write(format!(
                "    {}, {}, {}, {},\n",
                map_data.width,
                map_data.height,
                if map_data.has_bb { 1 } else { 0 },
                map_data.tile_changes.len()
            ));

            // tileset
            let tileset_index = writer.ident.get_asset_index(DataAssetType::Tileset, map_data.tileset_id)?;
            writer.write(format!("    &{}_tilesets[{}],\n", writer.ident.prefix_lower, tileset_index));

            // tile animation
            if let Some(tile_anim_id) = map_data.tile_anim_id {
                let tile_anim_index = writer.ident.get_asset_index(DataAssetType::TileAnimation, tile_anim_id)?;
                writer.write(format!("    &{}_tile_animations[{}],\n", writer.ident.prefix_lower, tile_anim_index));
            } else {
                writer.write("    NULL, // tile animation\n");
            }

            // tiles
            writer.write(format!("    {}_map_tiles_{},\n", writer.ident.prefix_lower, name_id));

            // tile changes
            if map_data.tile_changes.is_empty() {
                writer.write("    NULL, // tile changes\n");
            } else {
                writer.write(format!("    {}_map_tile_changes_{},\n", writer.ident.prefix_lower, name_id));
            }

            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_map_tile_change_names(writer: &ProjectDataWriter, map_data: &MapData, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_MAP_{}_TILE_CHANGE_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, change) in map_data.tile_changes.iter().enumerate() {
        let use_name_id = if change.name == "names" { "names0" } else { &change.name };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let change_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of tile change {} map {}", index, map_data.asset.id))
        })?;
        writer.write(format!(
            "  {}_MAP_{}_TILE_CHANGE_{},\n",
            writer.ident.prefix_upper,
            name_id_upper,
            change_name_id.to_ascii_uppercase()
        ));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_map_item_names(writer: &ProjectDataWriter, map_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === MAP TILE CHANGE NAMES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in map_ids.iter() {
        if let Some(map_data) = writer.store.assets.maps.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::MapData, *id)?;
            let name_id_upper = name_id.to_ascii_uppercase();
            if ! map_data.tile_changes.is_empty() {
                write_map_tile_change_names(writer, map_data, &name_id_upper)?;
            }
        }
    }

    Ok(())
}
