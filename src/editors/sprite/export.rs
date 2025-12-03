use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::image::ImageCollection;
use crate::data_asset::Sprite;

pub struct ExportDialog {
    pub open: bool,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub num_items_x: u32,
}

impl ExportDialog {
    pub fn new() -> Self {
        ExportDialog {
            open: false,
            filename: None,
            display_filename: None,
            num_items_x: 1,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_sprite_export")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        self.filename = None;
        self.display_filename = None;
        self.num_items_x = (sprite.num_frames as f32).sqrt().ceil() as u32;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if let Some(filename) = &self.filename {
            if let Err(e) = sprite.save_image_png(filename, self.num_items_x) {
                wc.open_message_box("Error Exporting", format!("Error exporting sprite to {}:\n{}", filename.display(), e));
            }
            true
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to export.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_export_sprite", sprite.asset.id)) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Export Sprite");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_export_grid", sprite.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("File name:");
                            ui.horizontal(|ui| {
                                if let Some(display_filename) = &self.display_filename {
                                    ui.add(egui::Label::new(display_filename).truncate());
                                } else {
                                    ui.label("");
                                }
                                if ui.button("...").clicked() {
                                    wc.sys_dialogs.save_file(
                                        Some(wc.egui.window),
                                        format!("editor_{}_export_sprite", sprite.asset.id),
                                        "Export Sprite",
                                        &[
                                            ("PNG files (*.png)", &["png"]),
                                            ("All files (*.*)", &["*"]),
                                        ]
                                    );
                                }
                            });
                            ui.end_row();

                            ui.label("Horiz Frames:");
                            ui.add(egui::Slider::new(&mut self.num_items_x, 1..=sprite.num_frames));
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() && self.confirm(wc, sprite) {
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
    }
}
