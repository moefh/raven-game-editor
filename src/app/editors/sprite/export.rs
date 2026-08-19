use crate::image::ImageCollectionIO;
use crate::data_asset::Sprite;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

pub struct ExportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub num_items_x: u32,
    pub export_sprite_sys_dlg_id: String,
}

impl ExportDialog {
    pub fn new() -> Self {
        ExportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_sprite_export"),
            num_items_x: 1,
            export_sprite_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        self.num_items_x = (sprite.num_frames as f32).sqrt().ceil() as u32;
        self.export_sprite_sys_dlg_id.replace_range(.., &format!("editor_{}_export_sprite", sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if let Err(e) = sprite.save_image_png(self.num_items_x).and_then(|data| file.write_data(data)) {
            wc.open_message_box("Error Exporting", format!("Error exporting sprite to {}:\n{}", file.filename(), e));
            false
        } else {
            true
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.export_sprite_sys_dlg_id) &&
            self.confirm(file, wc, sprite) {
                self.close(wc);
                return;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Export Sprite", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_export_grid", sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Horiz Frames:");
                        ui.add(egui::Slider::new(&mut self.num_items_x, 1..=sprite.num_frames));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Save File").clicked() {
                    wc.sys_dialogs.save_file(
                        Some(wc.egui.window),
                        self.export_sprite_sys_dlg_id.clone(),
                        "sprite",
                        "Export Sprite",
                        &[
                            ("PNG files (*.png)", &["png"]),
                            ("All files (*.*)", &["*"]),
                        ]
                    );
                }
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.close(wc);
        }
    }
}
