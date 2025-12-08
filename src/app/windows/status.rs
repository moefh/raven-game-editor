use crate::data_asset::{DataAssetId, DataAssetStore};

use super::AppWindow;
use super::super::WindowContext;

pub enum StatusWindowsTabs {
    Windows,
    Textures,
}

pub struct StatusWindow {
    pub base: AppWindow,
    window: Window,
}

impl StatusWindow {
    pub fn new(base: AppWindow) -> Self {
        StatusWindow {
            base,
            window: Window::new(),
        }
    }

    pub fn show(&mut self, wc: &WindowContext, store: &DataAssetStore) {
        let default_rect = egui::Rect {
            min: egui::Pos2::new(wc.window_space.max.x - 300.0, wc.window_space.min.y + 10.0),
            max: wc.window_space.max - egui::Vec2::new(10.0, 40.0),
        };
        let selected = wc.is_window_on_top(self.base.id);
        let title_bg = match wc.egui.ctx.theme() {
            egui::Theme::Light => if selected { egui::Color32::from_rgb(0xe0, 0xe0, 0xe0) } else { wc.egui.ctx.style().visuals.window_fill },
            egui::Theme::Dark => if selected { egui::Color32::from_rgb(0, 0x20, 0x40) } else { egui::Color32::from_rgb(0, 0x10, 0x20) },
        };
        let frame = egui::Frame::window(&wc.egui.ctx.style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: -2, bottom: -2 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 2 })
            .fill(title_bg);
        self.base.create_window(wc, "Editor Status", default_rect).frame(frame).show(wc.egui.ctx, |ui| {
            self.window.show(ui, wc, store);
        });
    }
}

struct Window {
    pub selected_tab: StatusWindowsTabs,
}

impl Window {
    fn new() -> Self {
        Window {
            selected_tab: StatusWindowsTabs::Windows,
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
                    asset_id: wc.window_tracker.editor_ids.get(&layer_id.id).copied(),
                    dialog: wc.window_tracker.open_ids.get(&layer_id.id).copied(),
                    is_non_asset_id: wc.window_tracker.non_asset_ids.contains(&layer_id.id),
                }
            }).collect()
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("editor_settings_main")
                    .num_columns(3)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for w in &window_list {
                            ui.label(format!("{:?}", w.window_id));
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
        egui::CentralPanel::default().show_inside(ui, |ui| {
            wc.egui.ctx.texture_ui(ui);
        });
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext, store: &DataAssetStore) {
        egui::TopBottomPanel::top("editor_status_window_tabs").show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, StatusWindowsTabs::Windows), "Windows").clicked() {
                    self.selected_tab = StatusWindowsTabs::Windows;
                }
                if ui.selectable_label(matches!(self.selected_tab, StatusWindowsTabs::Textures), "Textures").clicked() {
                    self.selected_tab = StatusWindowsTabs::Textures;
                }
            });
            ui.add_space(0.0);
        });

        egui::TopBottomPanel::bottom("editor_status_window_footer").show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label("");
        });

        match self.selected_tab {
            StatusWindowsTabs::Windows => { self.show_windows_tab(ui, wc, store) }
            StatusWindowsTabs::Textures => { self.show_textures_tab(ui, wc); }
        };
    }
}
