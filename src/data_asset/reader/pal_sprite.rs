use std::io::Result;

use super::{
    error,
    Value,
    AssetDef,
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

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<PalSprite> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::I16(width),
        Value::I16(height),
        Value::I16(num_frames),
        Value::U16(bits_per_pixel),
        Value::U8Array(palette),
        Value::ArrayRef(array)
    ] = &value[..] {
        let width = width.unsigned_abs() as u32;
        let height = height.unsigned_abs() as u32;
        let num_frames = num_frames.unsigned_abs() as u32;
        let bits_per_pixel = *bits_per_pixel as u32;
        let data = array.get_u8_array(project_data)?;
        let name = project_data.extract_asset_name("pal_sprite_data_", array)?;
        let color_to_palette_index_map = PalSprite::gen_color_to_palette_index_map(&palette.vec);
        Ok(PalSprite {
            asset: DataAsset::new(DataAssetType::PalSprite, asset_id, DataAsset::identifier_to_name(name)),
            width,
            height,
            num_frames,
            depth: PalSpriteDepth::from_bits_per_pixel(bits_per_pixel),
            palette: palette.vec.clone(),
            color_to_palette_index_map,
            data: ImageConverter::pal_image_to_pixels(data, &palette.vec, width, height, num_frames, bits_per_pixel),
        })
    } else {
        error(format!("bad pal_sprite data: {:?}", asset_def.value), asset_def.pos)
    }
}
