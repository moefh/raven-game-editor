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
    Sprite,
};

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData,
              image_converter: &super::ImageConverter) -> Result<Sprite> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U32(width),
        Value::U32(height),
        Value::U32(stride),
        Value::U32(num_frames),
        Value::ArrayRef(array)
    ] = &value[..] {
        let data = array.get_u32_array(project_data)?;
        let name = project_data.extract_asset_name("sprite_data_", array)?;

        let want_stride = width.div_ceil(4);
        if *stride != want_stride {
            error(format!("sprite stride doesn't match width: got {}, expected {}", stride, want_stride), asset_def.pos)?;
        }
        let want_len = stride * height * num_frames;
        if data.len() as u32 != want_len {
            error(format!("unexpected sprite data length: got {}, expected {} = {}*{}*{}",
                          data.len(), want_len, stride, height, num_frames), asset_def.pos)?;
        }
        let num_frames = if Sprite::MIRROR_FRAMES {
            if ! num_frames.is_multiple_of(2)  {
                error(format!("sprite with an odd number of tiles, should be even: {}", num_frames), asset_def.pos)?;
            }
            *num_frames / 2
        } else {
            *num_frames
        };

        Ok(Sprite {
            asset: DataAsset::new(DataAssetType::Sprite, asset_id, DataAsset::identifier_to_name(name)),
            width: *width,
            height: *height,
            num_frames,
            data: image_converter.get_image_pixels(data, *width, *height, num_frames),
        })
    } else {
        error(format!("bad sprite data: {:?}", asset_def.value), asset_def.pos)
    }
}
