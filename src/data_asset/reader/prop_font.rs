use std::io::Result;

use super::{
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    PropFont,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("height"), ValueDef::U8),
        (String::from("data"), ValueDef::ArrayRef),
        (String::from("char_widths"), ValueDef::U8Array),
        (String::from("char_offsets"), ValueDef::U16Array),
    ])
}

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

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<PropFont> {
    let height = asset_struct.get_u8("height")?;
    let data_array = asset_struct.get_array_ref("data")?;
    let char_widths = asset_struct.get_u8_array("char_widths")?;
    let char_offsets = asset_struct.get_u16_array("char_offsets")?;

    let height = height as u32;
    let data = data_array.get_u8_array(project_data)?;
    let name = project_data.extract_asset_name("prop_font_data_", data_array)?;

    Ok(PropFont {
        asset: DataAsset::new(DataAssetType::PropFont, asset_id, DataAsset::identifier_to_name(name)),
        max_width: 2 * height,
        height,
        data: bits_to_pixels(data, &char_widths.values, height, &char_offsets.values),
        char_widths: char_widths.values.clone(),
    })
}
