use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_tilesets(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.tilesets.cur_only.is_empty() && comparer.tilesets.other_only.is_empty() && comparer.tilesets.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.tilesets.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.tilesets.cur_only {
            if let Some(cur) = cur_store.assets.tilesets.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.tilesets.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.tilesets.other_only {
            if let Some(other) = other_store.assets.tilesets.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.tilesets.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.tilesets.diffs {
            if let Some(cur) = cur_store.assets.tilesets.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.tilesets.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {} tiles, other: {} tiles",
                            cur.num_tiles,
                            other.num_tiles
                        ));
                    });
                }
        }
    }

    diff_state
}
