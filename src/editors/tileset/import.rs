use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollectionIO, ImageSlicingMethod};
use crate::data_asset::Tileset;
use super::super::AssetEditorBase;

pub struct ImportDialog {
    pub open: bool,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub border: u32,
    pub space_between: u32,
}

impl ImportDialog {
    pub fn new() -> Self {
        ImportDialog {
            open: false,
            filename: None,
            display_filename: None,
            border: 0,
            space_between: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_import")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext) {
        self.filename = None;
        self.display_filename = None;
        self.border = 0;
        self.space_between = 0;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if let Some(filename) = &self.filename {
            let slicing = ImageSlicingMethod::by_size(Tileset::TILE_SIZE, Tileset::TILE_SIZE);
            match tileset.load_image_png(filename, &slicing, self.border, self.space_between) {
                Ok(()) => {
                    true
                }
                Err(e) => {
                    wc.logger.log(format!("ERROR reading file from {}:", filename.display()));
                    wc.logger.log(format!("{}", e));
                    wc.open_message_box(
                        "Error importing tileset",
                        "Error importing tileset file.\n\nConsult the log window for more information."
                    );
                    false
                }
            }
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to import.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_tileset", tileset.asset.id)) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        let mut confirmed = false;
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Import Tileset", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", tileset.asset.id))
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
                                wc.sys_dialogs.open_file(
                                    Some(wc.egui.window),
                                    format!("editor_{}_import_tileset", tileset.asset.id),
                                    "tileset",
                                    "Import Tileset",
                                    &[
                                        ("PNG files (*.png)", &["png"]),
                                        ("All files (*.*)", &["*"]),
                                    ]
                                );
                            }
                        });
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
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() && self.confirm(wc, tileset) {
                    confirmed = true;
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
        confirmed
    }
}
