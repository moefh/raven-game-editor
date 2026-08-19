use crate::image::{
    ImageCollectionIO,
    ImageSlicingMethod,
    ImageLoadOptions,
};
use crate::data_asset::Tileset;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

pub struct ImportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub import_tileset_sys_dlg_id: String,
    pub load_options: ImageLoadOptions,
}

impl ImportDialog {
    const DEFAULT_LOAD_OPTIONS: ImageLoadOptions = ImageLoadOptions {
        slicing_method: ImageSlicingMethod::by_size(Tileset::TILE_SIZE, Tileset::TILE_SIZE),
        border: 0,
        space_between: 0,
        zoom_x: 1,
        zoom_y: 1,
    };

    pub fn new() -> Self {
        ImportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_tileset_import"),
            load_options: Self::DEFAULT_LOAD_OPTIONS,
            import_tileset_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, tileset: &Tileset) {
        self.load_options = Self::DEFAULT_LOAD_OPTIONS;
        self.import_tileset_sys_dlg_id.replace_range(.., &format!("editor_{}_import_tileset", tileset.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if let Err(e) = file.read_data().and_then(|data| tileset.load_image_png(&data, &self.load_options)) {
            wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
            wc.logger.log(format!("{}", e));
            wc.open_message_box(
                "Error importing tileset",
                "Error importing tileset file.\n\nConsult the log window for more information."
            );
            false
        } else {
            true
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_tileset_sys_dlg_id) &&
            self.confirm(file, wc, tileset) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Import Tileset", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", tileset.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Zoom X:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_x, 1..=256));
                        ui.end_row();

                        ui.label("Zoom Y:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_y, 1..=256));
                        ui.end_row();

                        ui.label("Border:");
                        ui.add(egui::Slider::new(&mut self.load_options.border, 0..=32));
                        ui.end_row();

                        ui.label("Space between:");
                        ui.add(egui::Slider::new(&mut self.load_options.space_between, 0..=32));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Open File").clicked() {
                    wc.sys_dialogs.open_file(
                        Some(wc.egui.window),
                        self.import_tileset_sys_dlg_id.clone(),
                        "tileset",
                        "Import Tileset",
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
