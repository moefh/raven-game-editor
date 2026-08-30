use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_tile_anims(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.tile_anims.cur_only.is_empty() && comparer.tile_anims.other_only.is_empty() && comparer.tile_anims.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.tile_anims.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.tile_anims.cur_only {
            if let Some(cur) = cur_store.assets.tile_anims.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.tile_anims.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.tile_anims.other_only {
            if let Some(other) = other_store.assets.tile_anims.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.tile_anims.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.tile_anims.diffs {
            if let Some(cur) = cur_store.assets.tile_anims.get(&diff.cur_id) &&
                let Some(_other) = other_store.assets.tile_anims.get(&diff.other_id) {
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
