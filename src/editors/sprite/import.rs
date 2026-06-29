use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollectionIO, ImageSlicingMethod};
use crate::data_asset::Sprite;
use super::super::AssetEditorBase;

use super::super::ImageSlicingMethodOption;

pub struct ImportDialog {
    pub open: bool,
    pub filename: Option<PathBuf>,
    pub display_filename: Option<String>,
    pub slicing_method: ImageSlicingMethod,
    pub border: u32,
    pub space_between: u32,
}

impl ImportDialog {
    pub fn new() -> Self {
        ImportDialog {
            open: false,
            filename: None,
            display_filename: None,
            slicing_method: ImageSlicingMethod::by_number(1, 1),
            border: 0,
            space_between: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_sprite_import")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, _sprite: &Sprite) {
        self.filename = None;
        self.display_filename = None;
        self.slicing_method = ImageSlicingMethod::by_number(1, 1);
        self.border = 0;
        self.space_between = 0;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if let Some(filename) = &self.filename {
            match sprite.load_image_png(filename, &self.slicing_method, self.border, self.space_between) {
                Ok(()) => {
                    true
                }
                Err(e) => {
                    wc.logger.log(format!("ERROR reading file from {}:", filename.display()));
                    wc.logger.log(format!("{}", e));
                    wc.open_message_box(
                        "Error importing sprite",
                        "Error importing sprite file.\n\nConsult the log window for more information."
                    );
                    false
                }
            }
        } else {
            wc.open_message_box("Filename Needed", "You need to select a filename to import.");
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_sprite", sprite.asset.id)) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        let mut confirmed = false;
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Import Sprite", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", sprite.asset.id))
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
                                    format!("editor_{}_import_sprite", sprite.asset.id),
                                    "sprite",
                                    "Import Sprite",
                                    &[
                                        ("PNG files (*.png)", &["png"]),
                                        ("All files (*.*)", &["*"]),
                                    ]
                                );
                            }
                        });
                        ui.end_row();

                        ui.label("Slice image:");
                        let mut slicing_option = ImageSlicingMethodOption::from_slicing_method(&self.slicing_method);
                        egui::ComboBox::from_id_salt(format!("editor_{}_import_combo_slicing", sprite.asset.id))
                            .selected_text(slicing_option.text())
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut slicing_option,
                                                    ImageSlicingMethodOption::BySize,
                                                    ImageSlicingMethodOption::BySize.text());
                                ui.selectable_value(&mut slicing_option,
                                                    ImageSlicingMethodOption::ByNumber,
                                                    ImageSlicingMethodOption::ByNumber.text());
                            });
                        ui.end_row();
                        match slicing_option {
                            ImageSlicingMethodOption::BySize if ! matches!(self.slicing_method, ImageSlicingMethod::BySize{..}) => {
                                self.slicing_method = ImageSlicingMethod::by_size(sprite.width, sprite.height);
                            }
                            ImageSlicingMethodOption::ByNumber if ! matches!(self.slicing_method, ImageSlicingMethod::ByNumber{..}) => {
                                self.slicing_method = ImageSlicingMethod::by_number(1, 1);
                            }
                            _ => {}
                        }

                        match self.slicing_method {
                            ImageSlicingMethod::BySize { width, height } => {
                                let (mut w, mut h) = (width, height);
                                ui.label("Width:");  ui.add(egui::Slider::new(&mut w, 1..=256)); ui.end_row();
                                ui.label("Height:"); ui.add(egui::Slider::new(&mut h, 1..=256)); ui.end_row();
                                if w != width || h != height {
                                    self.slicing_method = ImageSlicingMethod::by_size(w, h);
                                }
                            }
                            ImageSlicingMethod::ByNumber { nx, ny } => {
                                let (mut x, mut y) = (nx, ny);
                                ui.label("Num X:"); ui.add(egui::Slider::new(&mut x, 1..=64)); ui.end_row();
                                ui.label("Num Y:"); ui.add(egui::Slider::new(&mut y, 1..=64)); ui.end_row();
                                if x != nx || y != ny {
                                    self.slicing_method = ImageSlicingMethod::by_number(x, y);
                                }
                            }
                        }

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
                if ui.button("Ok").clicked() && self.confirm(wc, sprite) {
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
