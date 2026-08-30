use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_fonts(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.fonts.cur_only.is_empty() && comparer.fonts.other_only.is_empty() && comparer.fonts.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.fonts.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.fonts.cur_only {
            if let Some(cur) = cur_store.assets.fonts.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.fonts.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.fonts.other_only {
            if let Some(other) = other_store.assets.fonts.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.fonts.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.fonts.diffs {
            if let Some(cur) = cur_store.assets.fonts.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.fonts.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {}x{}, other: {}x{}",
                            cur.width,
                            cur.height,
                            other.width,
                            other.height,
                        ));
                    });
                }
        }
    }

    diff_state
}
