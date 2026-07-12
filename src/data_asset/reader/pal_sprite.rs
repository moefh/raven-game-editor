use std::io::Result;

use super::{
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
    ImageConverter,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    PalSprite,
    PalSpriteDepth,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("width"), ValueDef::I16),
        (String::from("height"), ValueDef::I16),
        (String::from("num_frames"), ValueDef::I16),
        (String::from("bits_per_pixel"), ValueDef::U16),
        (String::from("palette"), ValueDef::U8Array),
        (String::from("data"), ValueDef::ArrayRef),
    ])
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<PalSprite> {
    let width = asset_struct.get_i16("width")?;
    let height = asset_struct.get_i16("height")?;
    let num_frames = asset_struct.get_i16("num_frames")?;
    let bits_per_pixel = asset_struct.get_u16("bits_per_pixel")?;
    let palette = asset_struct.get_u8_array("palette")?;
    let array = asset_struct.get_array_ref("data")?;

    let width = width.unsigned_abs() as u32;
    let height = height.unsigned_abs() as u32;
    let num_frames = num_frames.unsigned_abs() as u32;
    let bits_per_pixel = bits_per_pixel as u32;
    let data = array.get_u8_array(project_data)?;
    let name = project_data.extract_asset_name("pal_sprite_data_", array)?;
    let color_to_palette_index_map = PalSprite::gen_color_to_palette_index_map(&palette.values);
    Ok(PalSprite {
        asset: DataAsset::new(DataAssetType::PalSprite, asset_id, DataAsset::identifier_to_name(name)),
        width,
        height,
        num_frames,
        depth: PalSpriteDepth::from_bits_per_pixel(bits_per_pixel),
        palette: palette.values.clone(),
        color_to_palette_index_map,
        data: ImageConverter::pal_image_to_pixels(data, &palette.values, width, height, num_frames, bits_per_pixel),
    })
}
