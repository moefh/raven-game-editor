use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollectionIO, ImagePixelsCollection};
use crate::data_asset::Font;

pub struct ImportDialog {
    pub open: bool,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub width: u32,
    pub height: u32,
    pub border: u32,
    pub space_between: u32,
}

impl ImportDialog {
    pub fn new() -> Self {
        ImportDialog {
            open: false,
            filename: None,
            display_filename: None,
            width: 0,
            height: 0,
            border: 0,
            space_between: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_font_import")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, font: &Font) {
        self.filename = None;
        self.display_filename = None;
        self.width = font.width;
        self.height = font.height;
        self.border = 0;
        self.space_between = 0;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn fix_font_colors(font: &mut Font) {
        let bg = font.data[0];
        for color in font.data.iter_mut() {
            *color = if *color == bg { Font::BG_COLOR } else { Font::FG_COLOR };
        }
    }

    fn confirm(&mut self, wc: &mut WindowContext, font: &mut Font) -> bool {
        if let Some(filename) = &self.filename {
            let mut image = ImagePixelsCollection::new(1, 1, 1);
            match image.load_image_png(filename, self.width, self.height, self.border, self.space_between) {
                Ok(num_chars) => {
                    if num_chars == Font::NUM_CHARS {
                        font.width = self.width;
                        font.height = self.height;
                        font.data = std::mem::take(&mut image.data);
                        Self::fix_font_colors(font);
                        true
                    } else {
                        wc.open_message_box(
                            "Error importing font",
                            format!("Invalid font image: found {} characters, required {}.", num_chars, Font::NUM_CHARS),
                        );
                        false
                    }
                }
                Err(e) => {
                    wc.logger.log(format!("ERROR reading file from {}:", filename.display()));
                    wc.logger.log(format!("{}", e));
                    wc.open_message_box(
                        "Error importing font",
                        "Error importing font file.\n\nConsult the log window for more information."
                    );
                    false
                }
            }
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to import.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_font", font.asset.id)) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        let mut confirmed = false;
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Import Font");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_import_grid", font.asset.id))
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
                                        format!("editor_{}_import_font", font.asset.id),
                                        "Import Font",
                                        wc.path_library.entry("font"),
                                        &[
                                            ("PNG files (*.png)", &["png"]),
                                            ("All files (*.*)", &["*"]),
                                        ]
                                    );
                                }
                            });
                            ui.end_row();

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
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() && self.confirm(wc, font) {
                        confirmed = true;
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        confirmed
    }
}
