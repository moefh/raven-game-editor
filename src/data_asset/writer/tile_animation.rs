use std::io::Result;

use super::{
    ProjectDataWriter,
};
use super::super::{
    DataAssetId,
    DataAssetType,
};

pub fn write_tile_animations(writer: &ProjectDataWriter, tanim_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === TILE ANIMATIONS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    writer.log(format!("-> writing {} tile animations", writer.store.asset_ids.animations.len()));
    writer.write(format!(
        "const struct {}_TILE_ANIMATION {}_tile_animations[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in tanim_ids.iter() {
        if let Some(tanim) = writer.store.assets.tile_anims.get(id) {
            let parent_tileset_index = writer.ident.get_asset_index(DataAssetType::Tileset, tanim.parent_tileset_id)?;
            let anim_tileset_index = writer.ident.get_asset_index(DataAssetType::Tileset, tanim.anim_tileset_id)?;
            writer.write("  {\n");
            writer.write(format!("    &{}_tilesets[{}],  // parent\n", writer.ident.prefix_lower, parent_tileset_index));
            writer.write(format!("    &{}_tilesets[{}],  // animations\n", writer.ident.prefix_lower, anim_tileset_index));
            writer.write("    {");
            for (index, tloop) in tanim.loops.iter().enumerate() {
                if index.is_multiple_of(8) { writer.write("\n      "); }
                writer.write(format!("{{ {:3},{:3} }}, ", tloop.start, tloop.len));
            }
            writer.write("    }\n");
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
