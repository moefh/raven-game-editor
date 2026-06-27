use std::io::Result;

use super::{
    error,
    Value,
    AssetDef,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    PropFont,
};

fn bits_to_pixels(bits: &[u8], widths: &[u8], height: u32, offsets: &[u16]) -> Vec<u8> {
    let height = height as usize;
    let num_items = PropFont::NUM_CHARS as usize;
    let max_width = 2 * height;
    let mut pixels = vec![PropFont::BG_COLOR; max_width * height * num_items];
    if widths.len() != num_items || offsets.len() != widths.len() { return pixels; }
    for ch in 0..num_items {
        let offset = offsets[ch] as usize;
        let width = widths[ch] as usize;
        let stride = width.div_ceil(8);
        for y in 0..height {
            for x in 0..stride {
                let block = bits.get(offset + y*stride + x).map_or(0, |&v| v);
                for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                    if block & (1 << ix) != 0 && (x*8 + ix) < max_width {
                        pixels[((ch * height) + y) * max_width + x*8 + ix] = PropFont::FG_COLOR;
                    }
                }
            }
        }
    }
    pixels
}

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<PropFont> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U8(height),
        Value::ArrayRef(data_array),
        Value::U8Array(char_widths),
        Value::U16Array(char_offsets),
    ] = &value[..] {
        let height = *height as u32;
        let data = data_array.get_u8_array(project_data)?;
        let name = project_data.extract_asset_name("prop_font_data_", data_array)?;

        Ok(PropFont {
            asset: DataAsset::new(DataAssetType::PropFont, asset_id, DataAsset::identifier_to_name(name)),
            max_width: 2 * height,
            height,
            data: bits_to_pixels(data, &char_widths.vec, height, &char_offsets.vec),
            char_widths: char_widths.vec.clone(),
        })
    } else {
        error(format!("bad prop_font data : {:?}", asset_def.value), asset_def.pos)
    }
}
