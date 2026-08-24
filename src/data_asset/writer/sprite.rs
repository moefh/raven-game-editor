use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    Sprite,
};

fn write_sprite_frames(writer: &ProjectDataWriter, sprite: &Sprite, mirror: bool) {
    for frame_num in 0..sprite.num_frames {
        writer.write(format!("  // frame {}{}", frame_num, if mirror { " (mirror)" } else { "" }));
        writer.write_image_item(sprite.width, sprite.height, frame_num, &sprite.data, mirror);
    }
}

fn write_sprite_data(writer: &ProjectDataWriter, sprite: &Sprite, name_id: &str) {
    writer.write(format!(
        "static const uint32_t {}_sprite_data_{}[] = {{\n",
        writer.ident.prefix_lower,
        name_id
    ));
    write_sprite_frames(writer, sprite, false);
    if Sprite::MIRROR_FRAMES {
        write_sprite_frames(writer, sprite, true);
    }
    writer.write("};\n");
    writer.write("\n");
}

pub fn write_sprites(writer: &ProjectDataWriter, sprite_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === SPRITES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in sprite_ids.iter() {
        if let Some(sprite) = writer.store.assets.sprites.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Sprite, *id)?;
            write_sprite_data(writer, sprite, name_id);
        }
    }

    writer.log(format!("-> writing {} sprites", writer.store.asset_ids.sprites.len()));
    let mult_mirrors = if Sprite::MIRROR_FRAMES { 2 } else { 1 };
    writer.write(format!("const struct {}_IMAGE {}_sprites[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in sprite_ids.iter() {
        if let Some(sprite) = writer.store.assets.sprites.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Sprite, *id)?;
            writer.write(format!("  {{ {}, {}, {}, {}, {}_sprite_data_{} }},\n",
                sprite.width, sprite.height, sprite.width.div_ceil(4), sprite.num_frames * mult_mirrors,
                writer.ident.prefix_lower, name_id));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
