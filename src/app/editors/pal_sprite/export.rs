use std::path::PathBuf;

use crate::image::ImageCollectionIO;
use crate::data_asset::PalSprite;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
};

pub struct ExportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub num_items_x: u32,
    pub export_sys_dlg_id: String,
}

impl ExportDialog {
    pub fn new() -> Self {
        ExportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_sprite_export"),
            filename: None,
            display_filename: None,
            num_items_x: 1,
            export_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pal_sprite: &PalSprite) {
        self.filename = None;
        self.display_filename = None;
        self.num_items_x = (pal_sprite.num_frames as f32).sqrt().ceil() as u32;
        self.export_sys_dlg_id.replace_range(.., &format!("editor_{}_export_sprite", pal_sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        if let Some(filename) = &self.filename {
            if let Err(e) = pal_sprite.save_image_png(filename, self.num_items_x) {
                wc.open_message_box("Error Exporting", format!("Error exporting sprite to {}:\n{}", filename.display(), e));
            }
            true
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to export.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&self.export_sys_dlg_id) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Export Paletted Sprite", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_export_grid", pal_sprite.asset.id))
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
                                    self.export_sys_dlg_id.clone(),
                                    "sprite",
                                    "Export Paletted Sprite",
                                    &[
                                        ("PNG files (*.png)", &["png"]),
                                        ("All files (*.*)", &["*"]),
                                    ]
                                );
                            }
                        });
                        ui.end_row();

                        ui.label("Horiz Frames:");
                        ui.add(egui::Slider::new(&mut self.num_items_x, 1..=pal_sprite.num_frames));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() && self.confirm(wc, pal_sprite) {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_window_id, self.open);
        }
    }
}
