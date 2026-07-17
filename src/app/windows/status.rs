use crate::misc::IMAGES;
use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::WindowContext;

pub struct StatusWindow {
    pub base: AppWindowBase,
    content: StatusWindowContent,
}

impl StatusWindow {
    pub fn new(base: AppWindowBase) -> Self {
        StatusWindow {
            base,
            content: StatusWindowContent::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &DataAssetStore) -> AppWindowAction {
        let default_rect = egui::Rect {
            min: egui::Pos2::new(wc.window_space.max.x - 300.0, wc.window_space.min.y + 10.0),
            max: wc.window_space.max - egui::Vec2::new(10.0, 40.0),
        };
        self.base.show_window(wc, default_rect, [400.0, 300.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.info), "Editor Status");
            self.content.show(ui, wc, store);
            action
        })
    }
}

pub enum StatusWindowTab {
    Windows,
    Textures,
}

struct StatusWindowContent {
    pub selected_tab: StatusWindowTab,
}

impl StatusWindowContent {
    fn new() -> Self {
        StatusWindowContent {
            selected_tab: StatusWindowTab::Windows,
        }
    }

    fn show_windows_tab(&mut self, ui: &mut egui::Ui, wc: &WindowContext, store: &DataAssetStore) {
        struct WindowItem {
            window_id: egui::Id,
            layer: egui::layers::Order,
            asset_id: Option<DataAssetId>,
            dialog: Option<bool>,
            is_non_asset_id: bool,
        }
        let window_list: Vec<WindowItem> = wc.egui.ctx.memory(|mem| {
            mem.layer_ids().map(|layer_id| {
                WindowItem {
                    window_id: layer_id.id,
                    layer: layer_id.order,
                    asset_id: wc.window_tracker.editor_window_ids.get(&layer_id.id).copied(),
                    dialog: wc.window_tracker.open_dialog_ids.get(&layer_id.id).copied(),
                    is_non_asset_id: wc.window_tracker.non_editor_window_ids.contains(&layer_id.id),
                }
            }).collect()
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("editor_settings_main")
                    .num_columns(3)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for w in &window_list {
                            ui.add_sized(
                                [200.0, 20.0],
                                egui::Label::new(format!("{:?}", w.window_id))
                                    .show_tooltip_when_elided(false)
                                    .truncate()
                            );
                            ui.label(match w.layer {
                                egui::layers::Order::Background => "background",
                                egui::layers::Order::Middle => "middle",
                                egui::layers::Order::Foreground => "foreground",
                                egui::layers::Order::Tooltip => "tooltip",
                                egui::layers::Order::Debug => "debug",
                            });
                            match (w.asset_id, w.dialog, w.is_non_asset_id) {
                                (Some(asset_id), _, _) => {
                                    if let Some(asset) = store.assets.get_asset(asset_id) {
                                        ui.label(format!("<{}:{}> {}", asset.asset_type.name(), asset_id, asset.name));
                                    } else {
                                        ui.label(format!("<asset:{}> [NOT FOUND]", asset_id));
                                    }
                                }
                                (None, Some(true), _) => {
                                    ui.label("<open dialog>");
                                }
                                (None, Some(false), _) => {
                                    ui.label("<closed dialog>");
                                }
                                (None, None, true) => {
                                    ui.label("<non-asset window>");
                                }
                                (None, None, false) => {
                                    ui.label("");
                                }
                            }
                            ui.end_row();
                        }
                    })
            });
        });
    }

    fn show_textures_tab(&mut self, ui: &mut egui::Ui, wc: &WindowContext) {
        egui::CentralPanel::default().show(ui, |ui| {
            wc.egui.ctx.texture_ui(ui);
        });
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext, store: &DataAssetStore) {
        egui::Panel::top("editor_status_window_tabs").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, StatusWindowTab::Windows), "Windows").clicked() {
                    self.selected_tab = StatusWindowTab::Windows;
                }
                if ui.selectable_label(matches!(self.selected_tab, StatusWindowTab::Textures), "Textures").clicked() {
                    self.selected_tab = StatusWindowTab::Textures;
                }
            });
            ui.add_space(0.0);
        });
        match self.selected_tab {
            StatusWindowTab::Windows => { self.show_windows_tab(ui, wc, store) }
            StatusWindowTab::Textures => { self.show_textures_tab(ui, wc); }
        };
    }
}
