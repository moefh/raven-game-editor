use super::super::IMAGES;
use super::{SysDialogs, AppWindowTracker};

use egui_extras::{TableBuilder, Column};
use crate::data_asset::DataAssetStore;

pub struct AboutDialog {
    id: egui::Id,
    open: bool,
}

impl AboutDialog {
    const WINDOW_WIDTH: f32 = 400.0;
    const ICON_SIZE: egui::Vec2 = egui::Vec2::new(32.0, 32.0);

    pub fn new() -> Self {
        AboutDialog {
            id: egui::Id::new("dlg_about"),
            open: false,
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker) {
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        if ! self.open { return; }

        if egui::Modal::new(self.id).show(ui.ctx(), |ui| {
            sys_dialogs.block_ui(ui);
            ui.set_width(Self::WINDOW_WIDTH);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("About");
                ui.separator();
                ui.place(egui::Rect::from_min_size(ui.cursor().min + egui::Vec2::new(16.0, 16.0), Self::ICON_SIZE),
                         egui::Image::new(IMAGES.pico).max_size(Self::ICON_SIZE));
                ui.add_space(24.0);
                ui.heading("Raven Game Editor");
                ui.add_space(24.0);
                TableBuilder::new(ui)
                    .column(Column::exact(Self::WINDOW_WIDTH / 2.0).clip(true))
                    .column(Column::remainder())
                    .striped(false)
                    .vscroll(false)
                    .body(|mut body| {
                        body.row(16.0, |mut row| {
                            row.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label("Author:");
                            }); });
                            row.col(|ui| { ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.label("MoeFH");
                            }); });
                        });

                        body.row(16.0, |mut row| {
                            row.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label("Build date:");
                            }); });
                            row.col(|ui| { ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.label(env!("BUILD_DATE"));
                            }); });
                        });

                        body.row(16.0, |mut row| {
                            row.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label("File format version:");
                            }); });
                            row.col(|ui| { ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.label(format!("{} ({})", DataAssetStore::VERSION, DataAssetStore::VERSION_DATE));
                            }); });
                        });
                    });
                ui.add_space(8.0);
                ui.label("Source code:");
                ui.add_space(4.0);
                ui.hyperlink("https://github.com/moefh/raven-game-editor/");
                ui.hyperlink("https://codeberg.org/moefh/raven-game-editor/");
                ui.add_space(20.0);
                if ui.button("Close").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
    }
}
