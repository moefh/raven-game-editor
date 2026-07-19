use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, ModData};
use crate::data_asset::MOD_PERIOD_TABLE;

use super::AssetProblem;

pub fn check_merged_samples(store: &DataAssetStore) -> Vec<super::MergedSample> {
    let mut merged_samples = Vec::new();
    for (mod1_index, mod1_id) in store.asset_ids.mods.iter().enumerate() {
        if let Some(mod1_data) = store.assets.mods.get(mod1_id) {
            for mod2_id in store.asset_ids.mods.iter().skip(mod1_index+1) {
                if let Some(mod2_data) = store.assets.mods.get(mod2_id) {
                    for (sample1_index, sample1) in mod1_data.samples.iter().enumerate() {
                        if sample1.len == 0 || sample1.data.is_none() { continue; }
                        for (sample2_index, sample2) in mod2_data.samples.iter().enumerate() {
                            if sample2.len == 0 || sample2.data.is_none() { continue; }
                            if ModData::are_mod_samples_equal(sample1, sample2) {
                                merged_samples.push(super::MergedSample {
                                    saved_size: (sample2.len * (sample2.bits_per_sample/8) as u32) as usize,
                                    merged_mod_id: *mod2_id,
                                    merged_sample_index: sample2_index,
                                    data_mod_id: *mod1_id,
                                    data_sample_index: sample1_index,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    merged_samples
}

// Return None if the period exactly matches a period in the MOD
// table, or Some(the next nearest period in the table).
fn get_next_nearest_mod_period(period: u16) -> Option<u16> {
    for &table_period in MOD_PERIOD_TABLE {
        if period == table_period { return None; }
        if period > table_period {
            return Some(table_period);
        }
    }
    Some(0)
}

fn check_mod(mod_data: &ModData) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    let num_song_pos = mod_data.song_positions.iter().copied().max().unwrap_or(0) as usize + 1;
    let num_channels = mod_data.num_channels as usize;
    if num_song_pos * 64 * num_channels > mod_data.pattern.len() {
        problems.push(AssetProblem::ModPatternTooSmall { expected: num_song_pos * 64 * num_channels, got: mod_data.pattern.len() });
    }

    for song_pos in 0..num_song_pos {
        for row in 0..64 {
            for chan in 0..num_channels {
                let cell_index = (song_pos * 64 + row) * num_channels + chan;
                if let Some(cell) = mod_data.pattern.get(cell_index) &&
                    cell.period != 0 &&
                    let Some(period) = get_next_nearest_mod_period(cell.period) {
                        problems.push(AssetProblem::ModNoteOutOfTune {
                            song_position: song_pos as u32,
                            row: row as u32,
                            chan: chan as u8,
                            sharp_by: cell.period.saturating_sub(period),
                        });
                }
            }
        }
    }

    problems
}

pub fn check_mods(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for mod_data in store.assets.mods.iter() {
        asset_problems.insert(mod_data.asset.id, check_mod(mod_data));
    }
}
