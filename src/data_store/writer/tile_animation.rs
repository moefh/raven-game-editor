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
            let parent_tileset_ident = writer.ident.get_asset_ident(DataAssetType::Tileset, tanim.parent_tileset_id)?;
            let anim_tileset_ident = writer.ident.get_asset_ident(DataAssetType::Tileset, tanim.anim_tileset_id)?;
            writer.write("  {\n");
            writer.write(format!("    {},  // parent\n", parent_tileset_ident));
            writer.write(format!("    {},  // animations\n", anim_tileset_ident));
            writer.write("    {");
            for (index, tloop) in tanim.loops.iter().enumerate() {
                if index.is_multiple_of(8) { writer.write("\n     "); }
                writer.write(format!(" {{ {:3},{:3} }},", tloop.start, tloop.len));
            }
            writer.write("\n    }\n");
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
