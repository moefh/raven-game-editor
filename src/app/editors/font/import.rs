use crate::image::{
    ImageCollectionIO,
    ImagePixelsCollection,
    ImageSlicingMethod,
    ImageLoadOptions,
};
use crate::data_asset::Font;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

pub struct ImportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub import_sys_dlg_id: String,
    pub width: u32,
    pub height: u32,
    pub border: u32,
    pub space_between: u32,
}

impl ImportDialog {
    pub fn new() -> Self {
        ImportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_font_import"),
            import_sys_dlg_id: String::new(),
            width: 0,
            height: 0,
            border: 0,
            space_between: 0,
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, font: &Font) {
        self.width = font.width;
        self.height = font.height;
        self.border = 0;
        self.space_between = 0;
        self.import_sys_dlg_id.replace_range(.., &format!("editor_{}_import_font", font.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn fix_font_colors(font: &mut Font) {
        let bg = font.data[0];
        for color in font.data.iter_mut() {
            *color = if *color == bg { Font::BG_COLOR } else { Font::FG_COLOR };
        }
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, font: &mut Font) -> bool {
        let mut image = ImagePixelsCollection::new(1, 1, 1);
        let options = ImageLoadOptions {
            slicing_method: ImageSlicingMethod::by_size(self.width, self.height),
            space_between: self.space_between,
            border: self.border,
            zoom_x: 1,
            zoom_y: 1,
        };
        match file.read_data().and_then(|data| image.load_image_png(&data, &options)) {
            Ok(()) => {
                if image.num_items == Font::NUM_CHARS {
                    font.width = image.width;
                    font.height = image.height;
                    font.data = std::mem::take(&mut image.data);
                    Self::fix_font_colors(font);
                    true
                } else {
                    wc.open_message_box(
                        "Error importing font",
                        format!("Invalid font image: found {} characters, required {}.", image.num_items, Font::NUM_CHARS),
                    );
                    false
                }
            }
            Err(e) => {
                wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error importing font",
                    "Error importing font file.\n\nConsult the log window for more information."
                );
                false
            }
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sys_dlg_id) &&
            self.confirm(file, wc, font) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Import Font", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", font.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Char width:");
                        ui.add(egui::Slider::new(&mut self.width, 0..=256));
                        ui.end_row();

                        ui.label("Char height:");
                        ui.add(egui::Slider::new(&mut self.height, 0..=256));
                        ui.end_row();

                        ui.label("Border:");
                        ui.add(egui::Slider::new(&mut self.border, 0..=32));
                        ui.end_row();

                        ui.label("Space between:");
                        ui.add(egui::Slider::new(&mut self.space_between, 0..=32));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Open File").clicked() {
                    wc.sys_dialogs.open_file(
                        Some(wc.egui.window),
                        format!("editor_{}_import_font", font.asset.id),
                        "font",
                        "Import Font",
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
        false
    }
}
