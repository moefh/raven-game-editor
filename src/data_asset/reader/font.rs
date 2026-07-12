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
    Font,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("width"), ValueDef::U8),
        (String::from("height"), ValueDef::U8),
        (String::from("data"), ValueDef::ArrayRef),
    ])
}

fn bits_to_pixels(bits: &[u8], width: u32, height: u32) -> Vec<u8> {
    let stride = width.div_ceil(8) as usize;
    let mut pixels = vec![Font::BG_COLOR; (width * height * Font::NUM_CHARS) as usize];
    for y in 0..(height * Font::NUM_CHARS) as usize {
        for x in 0..stride {
            let block = bits.get(y*stride + x).map_or(0, |&v| v);
            for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                if block & (1 << ix) != 0 {
                    pixels[y * width as usize + x*8 + ix] = Font::FG_COLOR;
                }
            }
        }
    }
    pixels
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<Font> {
    let width = asset_struct.get_u8("width")?;
    let height = asset_struct.get_u8("height")?;
    let array = asset_struct.get_array_ref("data")?;

    let width = width as u32;
    let height = height as u32;
    let data = array.get_u8_array(project_data)?;
    let name = project_data.extract_asset_name("font_data_", array)?;
    Ok(Font {
        asset: DataAsset::new(DataAssetType::Font, asset_id, DataAsset::identifier_to_name(name)),
        width,
        height,
        data: bits_to_pixels(data, width, height),
    })
}
