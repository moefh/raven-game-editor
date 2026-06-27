use std::io::Result;

use super::{
    error,
    Value,
    AssetDef,
    ProjectData,
    TokenPosition,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    ModData,
    ModSample,
    ModCell,
};

fn create_samples(samples: &[Value], project_data: &ProjectData, pos: TokenPosition) -> Result<Vec<ModSample>> {
    if samples.len() != ModData::NUM_SAMPLES {
        return error(format!("invalid number of MOD samples: expected {}, found {}",
                             ModData::NUM_SAMPLES, samples.len()), pos);
    }

    let mut ret = Vec::new();
    for sample in samples.iter() {
        if let Value::Struct(value) = sample && let [
            Value::U32(len),
            Value::U32(loop_start),
            Value::U32(loop_len),
            Value::U8(finetune),
            Value::U8(volume),
            Value::U16(bits_per_sample),
            Value::ArrayRef(data),
        ] = &value[..] {
            ret.push(ModSample {
                len: *len,
                loop_start: *loop_start,
                loop_len: *loop_len,
                finetune: if *finetune > 7 { *finetune as i8 - 16 } else { *finetune as i8 },
                volume: *volume,
                bits_per_sample: *bits_per_sample,
                data: if data.is_null() { None } else { Some(data.get_i8_or_i16_array(project_data)?.take()) },
            });
        } else {
            return error(format!("bad MOD sample data: {:?}", sample), pos);
        }
    }
    Ok(ret)
}

fn create_pattern(num_patterns: u8, num_channels: u8, pattern: &[Value], pos: TokenPosition) -> Result<Vec<ModCell>> {
    if num_patterns as usize * num_channels as usize * 64 != pattern.len() {
        return error(format!("invalid pattern length: expected {}, found {}",
                             num_patterns as usize * num_channels as usize * 64, pattern.len()), pos);
    }

    let mut ret = Vec::new();
    for cell in pattern.iter() {
        if let Value::Struct(value) = cell && let [
            Value::U8(sample),
            Value::U8(note_index),
            Value::U16(effect),
        ] = &value[..] {
            let period = if *note_index == 0xff {
                0
            } else {
                ModData::get_note_period((*note_index % 12) as i32, (*note_index / 12) as i32)
            };
            ret.push(ModCell {
                sample: *sample,
                period,
                effect: *effect,
            });
        } else {
            return error(format!("invalid MOD pattern cell data: {:?}", cell), pos);
        }
    }

    Ok(ret)
}

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<ModData> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::Loop(samples),
        Value::U8(num_channels),
        Value::U8(num_song_positions),
        Value::U8Array(song_positions),
        Value::U8(num_patterns),
        Value::ArrayRef(pattern)
    ] = &value[..] {
        let name = project_data.extract_asset_name("mod_pattern_", pattern)?;
        let samples = create_samples(samples, project_data, asset_def.pos)?;
        let pattern = create_pattern(*num_patterns, *num_channels, pattern.get_struct_array(project_data)?, asset_def.pos)?;

        if *num_song_positions as usize != song_positions.vec.len() {
            return error(format!("invalid song positions array size: expected {}, got {}",
                                 *num_song_positions, song_positions.vec.len()), song_positions.pos);
        }

        Ok(ModData {
            asset: DataAsset::new(DataAssetType::ModData, asset_id, name.to_owned()),
            num_channels: *num_channels,
            samples,
            pattern,
            song_positions: song_positions.vec.clone(),
        })
    } else {
        error(format!("bad MOD data: {:?}", asset_def.value), asset_def.pos)
    }
}
