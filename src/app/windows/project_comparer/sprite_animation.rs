use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_animations(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.animations.cur_only.is_empty() && comparer.animations.other_only.is_empty() && comparer.animations.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.animations.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.animations.cur_only {
            if let Some(cur) = cur_store.assets.animations.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.animations.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.animations.other_only {
            if let Some(other) = other_store.assets.animations.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.animations.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.animations.diffs {
            if let Some(cur) = cur_store.assets.animations.get(&diff.cur_id) &&
                let Some(_other) = other_store.assets.animations.get(&diff.other_id) {
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
