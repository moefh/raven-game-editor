use std::io::Result;

use super::{
    error,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Tileset,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("width"), ValueDef::U32),
        (String::from("height"), ValueDef::U32),
        (String::from("stride"), ValueDef::U32),
        (String::from("num_frames"), ValueDef::U32),
        (String::from("data"), ValueDef::ArrayRef),
    ])
}

pub fn create(
    asset_id: DataAssetId,
    asset_struct: &ValueStruct,
    project_data: &ProjectData,
    image_converter: &super::ImageConverter
) -> Result<Tileset> {
    let width = asset_struct.get_u32("width")?;
    let height = asset_struct.get_u32("height")?;
    let stride = asset_struct.get_u32("stride")?;
    let num_tiles = asset_struct.get_u32("num_frames")?;
    let array = asset_struct.get_array_ref("data")?;

    let data = array.get_u32_array(project_data)?;
    let name = project_data.extract_asset_name("tileset_data_", array)?;

    let want_stride = width.div_ceil(4);
    if stride != want_stride {
        error(format!("tileset stride doesn't match width: got {}, expected {}", stride, want_stride), asset_struct.pos)?;
    }
    let want_len = stride * height * num_tiles;
    if data.len() as u32 != want_len {
        error(
            format!(
                "unexpected tileset data length: got {}, expected {} = {}*{}*{}",
                data.len(), want_len, stride, height, num_tiles
            ),
            asset_struct.pos
        )?;
    }

    Ok(Tileset {
        asset: DataAsset::new(DataAssetType::Tileset, asset_id, DataAsset::identifier_to_name(name)),
        width,
        height,
        num_tiles,
        data: image_converter.get_image_pixels(data, width, height, num_tiles),
    })
}
