use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    Tileset,
};

fn write_tileset_data(writer: &ProjectDataWriter, tileset: &Tileset, name_id: &str) {
    writer.write(format!(
        "static const uint32_t {}_tileset_data_{}[] = {{\n",
        writer.ident.prefix_lower,
        name_id
    ));
    for tile_num in 0..tileset.num_tiles {
        writer.write(format!("  // tile {}", tile_num));
        writer.write_image_item(tileset.width, tileset.height, tile_num, &tileset.data, false);
    }
    writer.write("};\n");
    writer.write("\n");
}

pub fn write_tilesets(writer: &ProjectDataWriter, tileset_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === TILESETS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in tileset_ids.iter() {
        if let Some(tileset) = writer.store.assets.tilesets.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Tileset, *id)?;
            write_tileset_data(writer, tileset, name_id);
        }
    }

    writer.log(format!("-> writing {} tilesets", writer.store.asset_ids.tilesets.len()));
    writer.write(format!("const struct {}_IMAGE {}_tilesets[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in tileset_ids.iter() {
        if let Some(tileset) = writer.store.assets.tilesets.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Tileset, *id)?;
            writer.write(format!(
                "  {{ {}, {}, {}, {}, {}_tileset_data_{} }},\n",
                tileset.width,
                tileset.height,
                tileset.width.div_ceil(4),
                tileset.num_tiles,
                writer.ident.prefix_lower,
                name_id
            ));
        }
    }
    writer.write("};\n");
    writer.write("\n");
    Ok(())
}
