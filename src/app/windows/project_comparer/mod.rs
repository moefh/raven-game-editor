mod tileset;
mod tile_animation;
mod map_data;
mod room;
mod world;
mod sprite;
mod pal_sprite;
mod sprite_animation;
mod mod_data;
mod sfx;
mod font;
mod prop_font;

use crate::include_ref_image;
use crate::misc::IMAGES;
use crate::misc::asset_defs::{
    ASSET_DEFS,
    get_asset_type_display_name,
};
use crate::data_asset::{
    self,
    DataAssetStore,
    DataAssetType,
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
}

pub struct ProjectComparerWindow {
    pub base: AppWindowBase,
    display: ProjectComparerDisplay,
}

impl ProjectComparerWindow {
    pub fn new(base: AppWindowBase) -> Self {
        ProjectComparerWindow {
            base,
            display: ProjectComparerDisplay::new(),
        }
    }

    pub fn clear(&mut self) {
        self.display.clear();
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        let default_rect = self.base.default_rect(wc, 700.0, 300.0);
        self.base.show_window(wc, default_rect, [300.0, 200.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.compare), "Project Comparer");
            let display_action = self.display.show(ui, wc, store);
            if matches!(action, AppWindowAction::None) {
                display_action
            } else {
                action
            }
        })
    }
}

enum DiffDisplay {
    Project,
    AssetType(DataAssetType),
}

impl DiffDisplay {
    fn is_asset_type(&self, asset_type: DataAssetType) -> bool {
        match self {
            DiffDisplay::AssetType(a) => { *a == asset_type }
            DiffDisplay::Project => { false }
        }
    }

    fn is_project(&self) -> bool {
        matches!(self, DiffDisplay::Project)
    }
}

struct ProjectComparerDisplay {
    other_store: Option<DataAssetStore>,
    other_name: Option<String>,
    comparer: ProjectComparer,
    diff_display: DiffDisplay,
    run_executed: bool,
}

impl ProjectComparerDisplay {
    const COMPARE_PROJECT_SYS_DLG_ID: &str = "compare_project";

    fn new() -> Self {
        ProjectComparerDisplay {
            other_name: None,
            other_store: None,
            comparer: ProjectComparer::new(),
            diff_display: DiffDisplay::Project,
            run_executed: false,
        }
    }

    pub fn clear(&mut self) {
        self.run_executed = false;
        self.comparer.clear();
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

    fn show_project_diff(
        &self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _store: &DataAssetStore,
        _other_store: &DataAssetStore
    ) -> AppWindowAction {
        if self.comparer.base_project.cur_data_size != self.comparer.base_project.other_data_size {
            ui.label(format!("Current project size: {}", self.comparer.base_project.cur_data_size));
            ui.label(format!("Other project size:   {}", self.comparer.base_project.other_data_size));
        } else {
            ui.label("Projects have the same size");
        }

        if ! self.comparer.has_any_asset_differences() {
            ui.add_space(20.0);
            ui.label("No asset differences found");
        }

        AppWindowAction::None
    }

    fn show_asset_type_diff(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &DataAssetStore,
        other_store: &DataAssetStore,
        asset_type: DataAssetType
    ) -> AppWindowAction {
        let diff_state = match asset_type {
            DataAssetType::Tileset => { tileset::show_tilesets(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::TileAnimation => { tile_animation::show_tile_anims(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::MapData => { map_data::show_maps(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::Room => { room::show_rooms(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::World => { world::show_worlds(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::Sprite => { sprite::show_sprites(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::PalSprite => { pal_sprite::show_pal_sprites(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::SpriteAnimation => { sprite_animation::show_animations(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::ModData => { mod_data::show_mods(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::Sfx => { sfx::show_sfxs(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::Font => { font::show_fonts(ui, wc, &self.comparer, store, other_store) }
            DataAssetType::PropFont => { prop_font::show_prop_fonts(ui, wc, &self.comparer, store, other_store) }
        };

        if ! diff_state.has_differences {
            if self.run_executed {
                ui.label("No differences found");
            } else {
                ui.horizontal(|ui| {
                    ui.label("(Current project changed -- click the button to compare)");
                });
            }
        }
        if let Some(asset_id) = diff_state.open_editor {
            AppWindowAction::ActivateAssetEditor(asset_id)
        } else {
            AppWindowAction::None
        }
    }

    fn run_compare(&mut self, store: &DataAssetStore) {
        if let Some(other) = &self.other_store {
            self.comparer.run(store, other);
            self.run_executed = true;
        }

        // if selected asset has no differences, select "Project"
        if let DiffDisplay::AssetType(asset_type) = self.diff_display && ! self.comparer.has_asset_differences(asset_type) {
            self.diff_display = DiffDisplay::Project;
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(Self::COMPARE_PROJECT_SYS_DLG_ID) {
            self.read_project_file(file, wc);
            self.run_compare(store);
        }

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
                    self.run_compare(store);
                }
            });
            ui.add_space(0.0);
        });

        egui::Panel::left("project_compare_asset_types").resizable(false).exact_size(180.0).show(ui, |ui| {
            ui.add_space(4.0);
            if self.other_store.is_some() {
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    let button = egui::Button::image_and_text(IMAGES.pico, "Project")
                        .frame_when_inactive(self.diff_display.is_project())
                        .gap(8.0);
                    if ui.add(button).clicked() {
                        self.diff_display = DiffDisplay::Project;
                    }

                    for asset_def in ASSET_DEFS {
                        if self.comparer.has_asset_differences(asset_def.asset_type) {
                            let label = get_asset_type_display_name(asset_def.asset_type).unwrap_or("?");
                            let button = egui::Button::image_and_text(include_ref_image!(asset_def.image), label)
                                .frame_when_inactive(self.diff_display.is_asset_type(asset_def.asset_type))
                                .gap(8.0);
                            if ui.add(button).clicked() {
                                self.diff_display = DiffDisplay::AssetType(asset_def.asset_type);
                            }
                        }
                    }
                    ui.add_space(2.0);
                });
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    if let Some(other_store) = &self.other_store {
                        match self.diff_display {
                            DiffDisplay::Project => { self.show_project_diff(ui, wc, store, other_store) }
                            DiffDisplay::AssetType(asset_type) => { self.show_asset_type_diff(ui, wc, store, other_store, asset_type) }
                        }
                    } else {
                        ui.label("No project to compare");
                        AppWindowAction::None
                    }
                }).inner
            }).inner
        }).inner
    }
}
