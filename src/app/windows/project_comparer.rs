use crate::misc::IMAGES;

use crate::data_asset::{
    self,
    DataAssetStore,
    DataAssetId,
};

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::{
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};
use super::super::project_comparer::ProjectComparer;

struct DiffState {
    has_differences: bool,
    open_editor: Option<DataAssetId>,
}

impl DiffState {
    const EMPTY:   Self = DiffState { has_differences: false, open_editor: None };
    const CHANGED: Self = DiffState { has_differences: true, open_editor: None};

    fn merge_open_editor(&mut self, asset_id: DataAssetId) {
        if self.open_editor.is_none() {
            self.open_editor = Some(asset_id);
        }
    }

    fn merge(&mut self, other: DiffState) {
        if ! self.has_differences && other.has_differences {
            self.has_differences = other.has_differences;
        }
        if let Some(asset_id) = other.open_editor {
            self.merge_open_editor(asset_id);
        }
    }
}

pub struct ProjectComparerWindow {
    pub base: AppWindowBase,
    other_store: Option<DataAssetStore>,
    other_name: Option<String>,
    comparer: ProjectComparer,
    pending_compare: bool,
}

impl ProjectComparerWindow {
    const COMPARE_PROJECT_SYS_DLG_ID: &str = "compare_project";

    pub fn new(base: AppWindowBase) -> Self {
        ProjectComparerWindow {
            base,
            other_name: None,
            other_store: None,
            comparer: ProjectComparer::new(),
            pending_compare: false,
        }
    }

    fn read_project_file(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext) {
        let filename = file.filename();
        wc.logger.log(format!("READING FILE TO COMPARE {}", filename));
        match file.read_string().and_then(|content| data_asset::deserialize_project(&content, wc.logger)) {
            Ok(store) => {
                wc.logger.log("DONE: project to compare read");
                self.other_name = Some(filename.to_owned());
                self.other_store = Some(store);
            },
            Err(e) => {
                wc.logger.log(format!("ERROR: {}", e));
                wc.open_message_box(
                    "Error Reading Project",
                    format!("Error reading project to compare: {}.\n\nConsult the log window for details.", e)
                );
            }
        }
    }

    fn show_tilesets(
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
        ui.label("=== TILESETS ==================");

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

    pub fn show(&mut self, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(Self::COMPARE_PROJECT_SYS_DLG_ID) {
            self.read_project_file(file, wc);
            self.pending_compare = true;
        }
        if self.pending_compare {
            self.pending_compare = false;
            if let Some(other) = &self.other_store {
                self.comparer.run(store, other);
            }
        }

        let default_rect = self.base.default_rect(wc, 700.0, 300.0);
        self.base.show_window(wc, default_rect, [300.0, 200.0], |ui, wc, base| {
            let mut action = base.show_title_bar(ui, Some(IMAGES.compare), "Project Comparer");
            egui::Panel::top("project_compare_toolbar").show(ui, |ui| {
                ui.add_space(1.0);
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::Button::image_and_text(IMAGES.open, "Read File").frame_when_inactive(false)
                    ).on_hover_text("Open file to compare").clicked() {
                        wc.sys_dialogs.open_file(
                            Some(wc.egui.window),
                            Self::COMPARE_PROJECT_SYS_DLG_ID.to_owned(),
                            "project",
                            "Select Project to Compare",
                            &[
                                ("Raven project files (*.h)", &["h"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    }
                    if ui.add_enabled(
                        self.other_store.is_some(),
                        egui::Button::image_and_text(IMAGES.compare, "Compare").frame_when_inactive(false)
                    ).on_hover_text("Compare project").clicked() {
                        self.pending_compare = true;
                        ui.ctx().request_repaint();
                    }
                });
                ui.add_space(0.0);
            });
            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(other_store) = &self.other_store {
                    let mut diff_state = DiffState::EMPTY;

                    diff_state.merge(Self::show_tilesets(ui, wc, &self.comparer, store, other_store));

                    if ! diff_state.has_differences {
                        ui.label("No differences found");
                    }
                    if matches!(action, AppWindowAction::None) && let Some(asset_id) = diff_state.open_editor {
                        action = AppWindowAction::ActivateAssetEditor(asset_id);
                    }
                } else {
                    ui.label("No project to compare");
                }
            });
            action
        })
    }
}
