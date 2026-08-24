use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    Font,
};

fn write_font_data(writer: &ProjectDataWriter, font: &Font, name: &str) {
    writer.write(format!("static const uint8_t {}_font_data_{}[] = {{\n", writer.ident.prefix_lower, name));
    for ch in 0..Font::NUM_CHARS {
        writer.write("  ");
        for y in 0..font.height {
            let mut byte = 0;
            for x in 0..font.width {
                let pixel = font.data[((ch * font.height + y) * font.width + x) as usize];
                byte |= if pixel == Font::FG_COLOR { 1<<(x%8) } else { 0 };
                if x % 8 == 7 {
                    writer.write(format!("{:#04x},", byte));
                    byte = 0;
                }
            }
            if ! font.width.is_multiple_of(8) {
                writer.write(format!("{:#04x},", byte));
            }
        }
        if (ch + Font::FIRST_CHAR) < 127 {
            writer.write(format!("  // '{}'\n", char::from_u32(ch + Font::FIRST_CHAR).unwrap_or('?')));
        } else {
            writer.write("  // DEL\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");
}

pub fn write_fonts(writer: &ProjectDataWriter, font_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === FONTS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in font_ids.iter() {
        if let Some(font) = writer.store.assets.fonts.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Font, *id)?;
            write_font_data(writer, font, name_id);
        }
    }

    writer.log(format!("-> writing {} fonts", writer.store.asset_ids.fonts.len()));
    writer.write(format!("const struct {}_FONT {}_fonts[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in font_ids.iter() {
        if let Some(font) = writer.store.assets.fonts.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Font, *id)?;
            writer.write(format!("  {{ {}, {}, {}_font_data_{} }},\n", font.width, font.height, writer.ident.prefix_lower, name_id));
        }
    }
    writer.write("};\n");
    writer.write("\n");
    Ok(())
}
