use std::io::Result;

use super::{
    Value,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
    ProjectDataReader,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Sfx,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("len"), ValueDef::U32),
        (String::from("loop_start"), ValueDef::U32),
        (String::from("loop_len"), ValueDef::U32),
        (String::from("bits_per_sample"), ValueDef::U16),
        (String::from("data"), ValueDef::Custom(custom_read_sfx_sample_data)),  // ArrayRef(i8/i16)
    ])
}

fn custom_read_sfx_sample_data(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.read_sample_data_ref()
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<Sfx> {
    let len = asset_struct.get_u32("len")?;
    let loop_start = asset_struct.get_u32("loop_start")?;
    let loop_len = asset_struct.get_u32("loop_len")?;
    let bits_per_sample = asset_struct.get_u16("bits_per_sample")?;
    let array = asset_struct.get_array_ref("data")?;

    let data = array.get_i8_or_i16_array(project_data)?;
    let name = project_data.extract_asset_name("sfx_samples_", array)?;
    Ok(Sfx {
        asset: DataAsset::new(DataAssetType::Sfx, asset_id, DataAsset::identifier_to_name(name)),
        len,
        loop_start,
        loop_len,
        bits_per_sample,
        samples: data.take(),
    })
}
