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
    MapData,
};

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<MapData> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U16(width),
        Value::U16(height),
        Value::U16(para_width),
        Value::U16(para_height),
        Value::AssetRef(tileset_ref),
        Value::ArrayRef(array)
    ] = &value[..] {
        let name = project_data.extract_asset_name("map_tiles_", array)?;
        let tiles = array.get_u8_array(project_data)?;

        let width = *width as u32;
        let height = *height as u32;
        let para_width = *para_width as u32;
        let para_height = *para_height as u32;

        let want_len = 3 * width * height + para_width * para_height;
        if tiles.len() as u32 != want_len {
            error(format!("unexpected tiles data length: got {}, expected {} = 3*{}*{} + {}*{}",
                          tiles.len(), want_len, width, height, para_width, para_height), array.pos)?;
        }

        let fg_size = (width * height) as usize;
        let fg_tiles = Vec::from(&tiles[0..fg_size]);
        let bg_tiles = Vec::from(&tiles[fg_size..2*fg_size]);
        let fx_tiles = Vec::from(&tiles[2*fg_size..3*fg_size]);
        let para_tiles = Vec::from(&tiles[3*fg_size..]);

        Ok(MapData {
            asset: DataAsset::new(DataAssetType::MapData, asset_id, DataAsset::identifier_to_name(name)),
            tileset_id: tileset_ref.get_asset_id(project_data)?,
            width,
            height,
            para_width,
            para_height,
            fg_tiles,
            bg_tiles,
            fx_tiles,
            para_tiles,
        })
    } else {
        error(format!("bad tileset data: {:?}", asset_def.value), asset_def.pos)
    }
}
