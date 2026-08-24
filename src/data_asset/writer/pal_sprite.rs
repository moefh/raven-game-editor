use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    PalSprite,
};

fn write_pal_sprite_frames(writer: &ProjectDataWriter, pal_sprite: &PalSprite) {
    let bits_per_pixel = pal_sprite.depth.bits_per_pixel();
    let stride = (pal_sprite.width * bits_per_pixel).div_ceil(8);
    let pixels_per_byte = 8 / bits_per_pixel;

    let mut src_index = 0;
    for frame_num in 0..pal_sprite.num_frames {
        writer.write(format!("  // frame {}", frame_num));
        for _ in 0..pal_sprite.height {
            writer.write("\n  ");
            for byte_num in 0..stride {
                let mut block = 0u8;
                for pix_num in 0..pixels_per_byte {
                    if byte_num * pixels_per_byte + pix_num >= pal_sprite.width { break; }
                    let color = pal_sprite.data[src_index];
                    src_index += 1;
                    block |= pal_sprite.color_to_palette_index(color) << (pix_num * bits_per_pixel);
                }
                writer.write(format!("{:#04x},", block));
            }
        }
        writer.write("\n");
    }
}

fn write_pal_sprite_data(writer: &ProjectDataWriter, pal_sprite: &PalSprite, name_id: &str) {
    writer.write(format!(
        "static const uint8_t {}_pal_sprite_data_{}[] = {{\n",
        writer.ident.prefix_lower,
        name_id
    ));
    write_pal_sprite_frames(writer, pal_sprite);
    writer.write("};\n");
    writer.write("\n");
}

pub fn write_pal_sprites(writer: &ProjectDataWriter, pal_sprite_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === PALETTED SPRITES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in pal_sprite_ids.iter() {
        if let Some(pal_sprite) = writer.store.assets.pal_sprites.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PalSprite, *id)?;
            write_pal_sprite_data(writer, pal_sprite, name_id);
        }
    }

    writer.log(format!("-> writing {} pal_sprites", writer.store.asset_ids.pal_sprites.len()));
    writer.write(format!("const struct {}_PAL_SPRITE {}_pal_sprites[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in pal_sprite_ids.iter() {
        if let Some(pal_sprite) = writer.store.assets.pal_sprites.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PalSprite, *id)?;
            writer.write("  {\n");
            writer.write(format!("    {}, {}, {}, {},\n",  pal_sprite.width, pal_sprite.height,
                pal_sprite.num_frames, pal_sprite.depth.bits_per_pixel()));
            writer.write("    { ");
            for color in &pal_sprite.palette {
                writer.write(format!("{:#04x},", color));
            }
            writer.write(" },\n");
            writer.write(format!("    {}_pal_sprite_data_{}\n",
                writer.ident.prefix_lower, name_id));
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
