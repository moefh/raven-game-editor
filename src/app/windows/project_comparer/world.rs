use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_worlds(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.worlds.cur_only.is_empty() && comparer.worlds.other_only.is_empty() && comparer.worlds.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.worlds.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.worlds.cur_only {
            if let Some(cur) = cur_store.assets.worlds.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.worlds.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.worlds.other_only {
            if let Some(other) = other_store.assets.worlds.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.worlds.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.worlds.diffs {
            if let Some(cur) = cur_store.assets.worlds.get(&diff.cur_id) &&
                let Some(_other) = other_store.assets.worlds.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                    });
                }
        }
    }

    diff_state
}
