use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    MapData,
};

fn write_map_tiles(writer: &ProjectDataWriter, tiles: &[u8]) {
    for (index, tile) in tiles.iter().enumerate() {
        if index.is_multiple_of(16) { writer.write("\n  "); }
        writer.write(format!("{:#04x},", tile))
    }
}

fn write_map_data(writer: &ProjectDataWriter, map_data: &MapData, name_id: &str) {
    writer.write(format!("static const uint8_t {}_map_tiles_{}[] = {{", writer.ident.prefix_lower, name_id));
    writer.write("\n  // foreground");
    write_map_tiles(writer, &map_data.fg_tiles);
    writer.write("\n  // background");
    write_map_tiles(writer, &map_data.bg_tiles);
    writer.write("\n  // effects");
    write_map_tiles(writer, &map_data.fx_tiles);
    writer.write("\n  // parallax");
    write_map_tiles(writer, &map_data.para_tiles);
    writer.write("\n};\n");
    writer.write("\n");
}

pub fn write_maps(writer: &ProjectDataWriter, map_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === MAPS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in map_ids.iter() {
        if let Some(map_data) = writer.store.assets.maps.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::MapData, *id)?;
            write_map_data(writer, map_data, name_id);
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
                map_data.para_width,
                map_data.para_height
            ));

            // tileset
            let tileset_index = writer.ident.get_asset_index(DataAssetType::Tileset, map_data.tileset_id)?;
            writer.write(format!("    &{}_tilesets[{}], // tileset\n", writer.ident.prefix_lower, tileset_index));

            // tile animation
            if let Some(tile_anim_id) = map_data.tile_anim_id {
                let tile_anim_index = writer.ident.get_asset_index(DataAssetType::TileAnimation, tile_anim_id)?;
                writer.write(format!("    &{}_tile_animations[{}], // tile animation\n", writer.ident.prefix_lower, tile_anim_index));
            } else {
                writer.write("    NULL,  // tile animation\n");
            }

            // tiles
            writer.write(format!("    {}_map_tiles_{}\n", writer.ident.prefix_lower, name_id));

            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
