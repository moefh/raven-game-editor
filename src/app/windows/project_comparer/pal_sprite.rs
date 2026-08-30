use crate::data_asset::{
    DataAssetStore,
};

use super::{
    WindowContext,
    ProjectComparer,
    DiffState,
};

pub fn show_pal_sprites(
    ui: &mut egui::Ui,
    _wc: &mut WindowContext,
    comparer: &ProjectComparer,
    cur_store: &DataAssetStore,
    other_store: &DataAssetStore
) -> DiffState {
    if comparer.pal_sprites.cur_only.is_empty() && comparer.pal_sprites.other_only.is_empty() && comparer.pal_sprites.diffs.is_empty() {
        return DiffState::EMPTY;
    }

    let mut diff_state = DiffState::CHANGED;

    if ! comparer.pal_sprites.cur_only.is_empty() {
        ui.label("-> only in current:");
        for id in &comparer.pal_sprites.cur_only {
            if let Some(cur) = cur_store.assets.pal_sprites.get(id) {
                ui.label(format!("   -> {}", cur.asset.name));
            }
        }
    }

    if ! comparer.pal_sprites.other_only.is_empty() {
        ui.label("-> only in other:");
        for id in &comparer.pal_sprites.other_only {
            if let Some(other) = other_store.assets.pal_sprites.get(id) {
                ui.label(format!("   -> {}", other.asset.name));
            }
        }
    }

    if ! comparer.pal_sprites.diffs.is_empty() {
        ui.label("-> different:");
        for diff in &comparer.pal_sprites.diffs {
            if let Some(cur) = cur_store.assets.pal_sprites.get(&diff.cur_id) &&
                let Some(other) = other_store.assets.pal_sprites.get(&diff.other_id) {
                    ui.horizontal(|ui| {
                        ui.label("   -> ");
                        if ui.button(&cur.asset.name).clicked() {
                            diff_state.merge_open_editor(diff.cur_id);
                        }
                        ui.label(format!(
                            "current: {} frames, other: {} frames",
                            cur.num_frames,
                            other.num_frames
                        ));
                    });
                }
        }
    }

    diff_state
}
