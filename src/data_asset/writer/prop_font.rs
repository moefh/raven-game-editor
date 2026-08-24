use std::io::{Result, Error};
use std::collections::HashMap;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    PropFont,
};

fn write_prop_font_data(writer: &ProjectDataWriter, font: &PropFont, name_id: &str) -> Vec::<u16> {
    writer.write(format!("static const uint8_t {}_prop_font_data_{}[] = {{\n", writer.ident.prefix_lower, name_id));
    let mut char_offset = 0;
    let mut char_offsets = Vec::<u16>::new();
    for ch in 0..PropFont::NUM_CHARS {
        char_offsets.push(char_offset);
        writer.write("  ");
        for y in 0..font.height {
            let width = font.char_widths[ch as usize] as u32;
            let mut byte = 0;
            for x in 0..width {
                let pixel = font.data[((ch * font.height + y) * font.max_width + x) as usize];
                byte |= if pixel == PropFont::FG_COLOR { 1<<(x%8) } else { 0 };
                if x % 8 == 7 {
                    writer.write(format!("{:#04x},", byte));
                    byte = 0;
                    char_offset += 1;
                }
            }
            if ! width.is_multiple_of(8) {
                writer.write(format!("{:#04x},", byte));
                char_offset += 1;
            }
        }
        if (ch + PropFont::FIRST_CHAR) < 127 {
            writer.write(format!("  // '{}'\n", char::from_u32(ch + PropFont::FIRST_CHAR).unwrap_or('?')));
        } else {
            writer.write("  // DEL\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");
    char_offsets
}

pub fn write_prop_fonts(writer: &ProjectDataWriter, prop_font_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === PROPORTIONAL FONTS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    let mut font_char_offsets = HashMap::<DataAssetId, Vec<u16>>::new();
    for id in prop_font_ids.iter() {
        if let Some(font) = writer.store.assets.prop_fonts.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PropFont, *id)?;
            let char_offsets = write_prop_font_data(writer, font, name_id);
            font_char_offsets.insert(*id, char_offsets);
        }
    }

    writer.log(format!("-> writing {} prop fonts", writer.store.asset_ids.prop_fonts.len()));
    writer.write(format!("const struct {}_PROP_FONT {}_prop_fonts[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in prop_font_ids.iter() {
        if let Some(font) = writer.store.assets.prop_fonts.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PropFont, *id)?;
            let char_offsets = font_char_offsets.get(id).ok_or_else(|| {
                Error::other(format!("can't find char offset for pfont '{}'", id))
            })?;
            writer.write("  {\n");
            writer.write(format!("    {},\n", font.height));
            writer.write(format!("    {}_prop_font_data_{},\n", writer.ident.prefix_lower, name_id));
            writer.write("    {  // char widths");
            for ch in 0..PropFont::NUM_CHARS {
                if ch.is_multiple_of(24) { writer.write("\n      "); }
                writer.write(format!("{},", font.char_widths[ch as usize]));
            }
            writer.write("\n    },\n");
            writer.write("    {  // char offsets");
            for ch in 0..PropFont::NUM_CHARS {
                if ch.is_multiple_of(8) { writer.write("\n      "); }
                writer.write(format!("{:>4},", char_offsets[ch as usize]));
            }
            writer.write("\n    }\n");
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");
    Ok(())
}
