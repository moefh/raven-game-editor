use std::io::Result;

use super::{
    error,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Sprite,
};

pub fn create(
    asset_id: DataAssetId,
    asset_struct: &ValueStruct,
    project_data: &ProjectData,
    image_converter: &super::ImageConverter
) -> Result<Sprite> {
    let width = asset_struct.get_u32("width")?;
    let height = asset_struct.get_u32("height")?;
    let stride = asset_struct.get_u32("stride")?;
    let num_frames = asset_struct.get_u32("num_frames")?;
    let array = asset_struct.get_array_ref("data")?;

    let data = array.get_u32_array(project_data)?;
    let name = project_data.extract_asset_name("sprite_data_", array)?;

    let want_stride = width.div_ceil(4);
    if stride != want_stride {
        error(format!("sprite stride doesn't match width: got {}, expected {}", stride, want_stride), asset_struct.pos)?;
    }
    let want_len = stride * height * num_frames;
    if data.len() as u32 != want_len {
        error(format!("unexpected sprite data length: got {}, expected {} = {}*{}*{}",
            data.len(), want_len, stride, height, num_frames), asset_struct.pos)?;
    }
    let num_frames = if Sprite::MIRROR_FRAMES {
        if ! num_frames.is_multiple_of(2)  {
            error(format!("sprite with an odd number of tiles, should be even: {}", num_frames), asset_struct.pos)?;
        }
        num_frames / 2
    } else {
        num_frames
    };

    Ok(Sprite {
        asset: DataAsset::new(DataAssetType::Sprite, asset_id, DataAsset::identifier_to_name(name)),
        width,
        height,
        num_frames,
        data: image_converter.get_image_pixels(data, width, height, num_frames),
    })
}
