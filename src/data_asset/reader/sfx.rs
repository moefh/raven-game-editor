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
    Sfx,
};

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<Sfx> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U32(len),
        Value::U32(loop_start),
        Value::U32(loop_len),
        Value::U16(bits_per_sample),
        Value::ArrayRef(array)
    ] = &value[..] {
        let data = array.get_i8_or_i16_array(project_data)?;
        let name = project_data.extract_asset_name("sfx_samples_", array)?;
        Ok(Sfx {
            asset: DataAsset::new(DataAssetType::Sfx, asset_id, DataAsset::identifier_to_name(name)),
            len: *len,
            loop_start: *loop_start,
            loop_len: *loop_len,
            bits_per_sample: *bits_per_sample,
            samples: data.take(),
        })
    } else {
        error(format!("bad sfx data: {:?}", asset_def.value), asset_def.pos)
    }
}
