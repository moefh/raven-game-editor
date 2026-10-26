use std::io::Result;

use super::{
    error,
    Value,
    ValueDef,
    ValueDefStruct,
    ValueArray,
    ValueStruct,
    ProjectData,
    ProjectDataReader,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    ModData,
    ModSample,
    ModCell,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("samples"), ValueDef::StructArray(  // 31 samples
            ValueDefStruct::new(vec![
                (String::from("len"), ValueDef::U32),
                (String::from("loop_start"), ValueDef::U32),
                (String::from("loop_len"), ValueDef::U32),
                (String::from("finetune"), ValueDef::U8),
                (String::from("volume"), ValueDef::U8),
                (String::from("bits_per_sample"), ValueDef::U16),
                (String::from("data"), ValueDef::Custom(custom_read_mod_sample_data)),  // ArrayRef(i8/i16)
            ])
        )),
        (String::from("num_channels"), ValueDef::U8),
        (String::from("num_song_positions"), ValueDef::U8),
        (String::from("song_positions"), ValueDef::U8Array),
        (String::from("num_patterns"), ValueDef::U8),
        (String::from("patterns"), ValueDef::ArrayRef),
    ])
}

pub fn get_global_struct_defs() -> Vec<(String, ValueDefStruct)> {
    vec![
        (String::from("MOD_CELL"), ValueDefStruct::new(vec![
            (String::from("sample"), ValueDef::U8),
            (String::from("note_index"), ValueDef::U8),
            (String::from("effect"), ValueDef::U16),
        ])),
    ]
}

fn custom_read_mod_sample_data(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.read_sample_data_ref()
}

fn create_samples(samples: &ValueArray<ValueStruct>, project_data: &ProjectData) -> Result<Vec<ModSample>> {
    if samples.values.len() != ModData::NUM_SAMPLES {
        return error(
            format!("invalid number of MOD samples: expected {}, found {}", ModData::NUM_SAMPLES, samples.values.len()),
            samples.pos
        );
    }

    let mut ret = Vec::new();
    for sample in samples.values.iter() {
        let len = sample.get_u32("len")?;
        let loop_start = sample.get_u32("loop_start")?;
        let loop_len = sample.get_u32("loop_len")?;
        let finetune = sample.get_u8("finetune")?;
        let volume = sample.get_u8("volume")?;
        let bits_per_sample = sample.get_u16("bits_per_sample")?;
        let data = sample.get_array_ref("data")?;

        let finetune = finetune.clamp(0, i8::MAX as u8) as i8;
        let data = if data.is_null() { None } else { Some(data.get_i8_or_i16_array(project_data)?.take()) };

        ret.push(ModSample {
            len,
            loop_start,
            loop_len,
            finetune: if finetune > 7 { finetune - 16 } else { finetune },
            volume,
            bits_per_sample,
            data,
        });
    }
    Ok(ret)
}

fn create_pattern(num_patterns: u8, num_channels: u8, pattern: &ValueArray<ValueStruct>) -> Result<Vec<ModCell>> {
    if num_patterns as usize * num_channels as usize * 64 != pattern.values.len() {
        return error(
            format!(
                "invalid pattern length: expected {}, found {}",
                num_patterns as usize * num_channels as usize * 64, pattern.values.len()),
            pattern.pos
        );
    }

    let mut ret = Vec::new();
    for cell in pattern.values.iter() {
        let sample = cell.get_u8("sample")?;
        let note_index = cell.get_u8("note_index")?;
        let effect = cell.get_u16("effect")?;

        let period = if note_index == 0xff {
            0
        } else {
            ModData::get_note_period((note_index % 12) as i32, (note_index / 12) as i32)
        };
        ret.push(ModCell {
            sample,
            period,
            effect,
        });
    }

    Ok(ret)
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<ModData> {
    let samples = asset_struct.get_struct_array("samples")?;
    let num_channels = asset_struct.get_u8("num_channels")?;
    let num_song_positions = asset_struct.get_u8("num_song_positions")?;
    let song_positions = asset_struct.get_u8_array("song_positions")?;
    let num_patterns = asset_struct.get_u8("num_patterns")?;
    let pattern = asset_struct.get_array_ref("patterns")?;

    let name = project_data.extract_asset_name("mod_pattern_", pattern)?;
    let samples = create_samples(samples, project_data)?;
    let pattern = create_pattern(num_patterns, num_channels, pattern.get_struct_array(project_data)?)?;

    if num_song_positions as usize != song_positions.values.len() {
        return error(
            format!("invalid song positions array size: expected {}, got {}", num_song_positions, song_positions.values.len()),
            song_positions.pos
        );
    }

    Ok(ModData {
        asset: DataAsset::new(DataAssetType::ModData, asset_id, name.to_owned()),
        num_channels,
        samples,
        pattern,
        song_positions: song_positions.values.clone(),
    })
}
