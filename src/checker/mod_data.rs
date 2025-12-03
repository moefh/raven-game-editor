use crate::data_asset::{AssetCollection, ModData};
use crate::data_asset::MOD_PERIOD_TABLE;

use super::AssetProblem;

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

pub fn check_mod(mod_data: &ModData, _assets: &AssetCollection) -> Vec<AssetProblem> {
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
