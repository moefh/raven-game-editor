use crate::image::{
    ImageCollectionIO,
    ImagePixelsCollection,
};
use crate::data_store::{
    Sprite,
};

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
    SelectImageLocation,
};

pub struct ExportSelectedFramesDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub dlg_id_insert_location_combo: egui::Id,
    pub selected_frame: u32,
    pub start_frame_location: SelectImageLocation,
    pub num_frames: u32,
    pub num_frames_x: u32,
    pub export_sys_dlg_id: String,
}

impl ExportSelectedFramesDialog {
    pub fn new() -> Self {
        ExportSelectedFramesDialog {
            open: false,
            dlg_window_id: egui::Id::unique("dlg_sprite_export_sel_frames"),
            dlg_id_insert_location_combo: egui::Id::unique("dlg_sprite_export_sel_frames_combo"),
            selected_frame: 0,
            start_frame_location: SelectImageLocation::Selected,
            num_frames: 1,
            num_frames_x: 1,
            export_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, selected_frame: u32, sprite: &Sprite) {
        self.selected_frame = selected_frame;
        self.num_frames = 1;
        self.num_frames_x = 1;
        self.start_frame_location = SelectImageLocation::Selected;
        self.export_sys_dlg_id.replace_range(.., &format!("editor_{}_export_sel_frames", sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, sprite: &Sprite) -> bool {
        let first_frame = match self.start_frame_location {
            SelectImageLocation::Start => { 0 }
            SelectImageLocation::Selected => { self.selected_frame }
        };
        if first_frame + self.num_frames > sprite.num_frames {
            wc.open_message_box("Error Exporting", "Invalid number of frames to export");
            return false;
        }

        let frame_len: usize = (sprite.width * sprite.height) as usize;
        let mut export_image = ImagePixelsCollection::new(sprite.width, sprite.height, self.num_frames);
        let src_start = first_frame as usize * frame_len;
        let copy_len = self.num_frames as usize * frame_len;
        export_image.data[0..copy_len].copy_from_slice(&sprite.data[src_start .. src_start+copy_len]);
        if let Err(e) = export_image.save_image_png(self.num_frames_x).and_then(|data| file.write_data(data)) {
            wc.open_message_box("Error Exporting", format!("Error exporting selected frames to {}:\n{}", file.filename(), e));
            false
        } else {
            true
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.export_sys_dlg_id) &&
            self.confirm(file, wc, sprite) {
                self.close(wc);
                return;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 300.0, "Export Frames", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_export_grid", sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Start at:");
                        egui::ComboBox::from_id_salt(self.dlg_id_insert_location_combo)
                            .selected_text(self.start_frame_location.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.start_frame_location,
                                    SelectImageLocation::Start,
                                    SelectImageLocation::Start.text()
                                );
                                ui.selectable_value(
                                    &mut self.start_frame_location,
                                    SelectImageLocation::Selected,
                                    SelectImageLocation::Selected.text()
                                );
                            });
                        ui.end_row();

                        let max_total_frames = match self.start_frame_location {
                            SelectImageLocation::Start => { sprite.num_frames }
                            SelectImageLocation::Selected => { sprite.num_frames.saturating_sub(self.selected_frame).max(1) }
                        };
                        ui.label("Total Frames:");
                        ui.add(egui::Slider::new(&mut self.num_frames, 1..=max_total_frames));
                        ui.end_row();

                        ui.label("Horiz Frames:");
                        ui.add(egui::Slider::new(&mut self.num_frames_x, 1..=self.num_frames));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Save File").clicked() {
                    wc.sys_dialogs.save_file(
                        Some(wc.egui.window),
                        self.export_sys_dlg_id.clone(),
                        "sprite",
                        "Export Frames",
                        &sprite.asset.name,
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
