use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_rooms(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.rooms.cur_only.is_empty() && comparer.rooms.other_only.is_empty() && comparer.rooms.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.rooms.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.rooms.cur_only {
            if let Some(cur) = cur_store.assets.rooms.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.rooms.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.rooms.other_only {
            if let Some(other) = other_store.assets.rooms.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.rooms.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.rooms.diffs {
            if let Some(cur) = cur_store.assets.rooms.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.rooms.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {} triggers, other: {} triggers",
                            cur.triggers.len(),
                            other.triggers.len()
                        ));
                    });
                }
        }
    }

    diff_state
}
