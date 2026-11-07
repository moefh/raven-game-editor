use crate::data_store::{
    DataStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_enemies(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataStore,
    other_store: &DataStore
) -> DiffState {
    if comparer.enemies.cur_only.is_empty() && comparer.enemies.other_only.is_empty() && comparer.enemies.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.enemies.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.enemies.cur_only {
            if let Some(cur) = cur_store.assets.enemies.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.enemies.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.enemies.other_only {
            if let Some(other) = other_store.assets.enemies.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.enemies.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.enemies.diffs {
            if let Some(cur) = cur_store.assets.enemies.get(&diff.cur_id) &&
                let Some(_other) = other_store.assets.enemies.get(&diff.other_id) {
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
