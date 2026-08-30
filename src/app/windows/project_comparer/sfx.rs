use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_sfxs(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.sfxs.cur_only.is_empty() && comparer.sfxs.other_only.is_empty() && comparer.sfxs.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.sfxs.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.sfxs.cur_only {
            if let Some(cur) = cur_store.assets.sfxs.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.sfxs.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.sfxs.other_only {
            if let Some(other) = other_store.assets.sfxs.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.sfxs.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.sfxs.diffs {
            if let Some(cur) = cur_store.assets.sfxs.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.sfxs.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {} samples, other: {} samples",
                            cur.samples.len(),
                            other.samples.len()
                        ));
                    });
                }
        }
    }

    diff_state
}
