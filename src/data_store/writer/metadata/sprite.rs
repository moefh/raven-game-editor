use std::collections::HashMap;
use std::io::Result;

use super::{
    write_array,
    ProjectDataWriter,
    SpriteMetadata,
};
use super::super::{
    IdentStore,
};
use super::super::super::{
    DataAssetId,
    DataAssetIdList,
};

fn write_sprite(writer: &ProjectDataWriter, sprite_md: Option<&SpriteMetadata>) {
    if let Some(sprite_md) = sprite_md && ! sprite_md.frames.is_empty() {
        writer.write("  { {\n");
        for frame in &sprite_md.frames {
            writer.write("    {");
            writer.write(format!(" {}, ", IdentStore::lower_cleanup(&frame.name)));

            writer.write("{ ");
            write_array(writer, &frame.indices);
            writer.write(" }");
            writer.write(" },\n");
        }
        writer.write("  } },\n");
    } else {
        writer.write("  { {} },\n");
    }
}

pub fn write(
    writer: &ProjectDataWriter,
    sprite_ids: &DataAssetIdList,
    sprites: &HashMap<DataAssetId, super::SpriteMetadata>
) -> Result<()> {
    writer.write(format!(
        "const struct {}_SPRITE_METADATA {}_sprite_metadata[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
    ));
    for sprite_id in sprite_ids.iter() {
        write_sprite(writer, sprites.get(sprite_id));
    }
    writer.write("};\n");
    Ok(())
}
