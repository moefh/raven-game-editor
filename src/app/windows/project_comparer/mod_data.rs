use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_mods(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.mods.cur_only.is_empty() && comparer.mods.other_only.is_empty() && comparer.mods.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.mods.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.mods.cur_only {
            if let Some(cur) = cur_store.assets.mods.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.mods.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.mods.other_only {
            if let Some(other) = other_store.assets.mods.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.mods.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.mods.diffs {
            if let Some(cur) = cur_store.assets.mods.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.mods.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {} song positions, other: {} song positions",
                            cur.song_positions.len(),
                            other.song_positions.len()
                        ));
                    });
                }
        }
    }

    diff_state
}
