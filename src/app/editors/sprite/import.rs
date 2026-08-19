use crate::image::{
    ImageCollectionIO,
    ImageSlicingMethod,
    ImageLoadOptions,
};
use crate::data_asset::Sprite;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

use super::super::ImageSlicingMethodOption;

pub struct ImportDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub load_options: ImageLoadOptions,
    pub import_sprite_sys_dlg_id: String,
}

impl ImportDialog {
    const DEFAULT_LOAD_OPTIONS: ImageLoadOptions = ImageLoadOptions {
        slicing_method: ImageSlicingMethod::by_number(1, 1),
        border: 0,
        space_between: 0,
        zoom_x: 1,
        zoom_y: 1,
    };

    pub fn new() -> Self {
        ImportDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_sprite_import"),
            load_options: Self::DEFAULT_LOAD_OPTIONS,
            import_sprite_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sprite: &Sprite) {
        self.load_options = Self::DEFAULT_LOAD_OPTIONS;
        self.import_sprite_sys_dlg_id.replace_range(.., &format!("editor_{}_import_sprite", sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if let Err(e) = file.read_data().and_then(|data| sprite.load_image_png(&data, &self.load_options)) {
            wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
            wc.logger.log(format!("{}", e));
            wc.open_message_box(
                "Error importing sprite",
                "Error importing sprite file.\n\nConsult the log window for more information."
            );
            false
        } else {
            true
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sprite_sys_dlg_id) &&
            self.confirm(file, wc, sprite) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Import Sprite", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Zoom X:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_x, 1..=256));
                        ui.end_row();

                        ui.label("Zoom Y:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_y, 1..=256));
                        ui.end_row();

                        ui.label("Slice image:");
                        let mut slicing_option = ImageSlicingMethodOption::from_slicing_method(&self.load_options.slicing_method);
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
                            ImageSlicingMethodOption::BySize if ! matches!(
                                self.load_options.slicing_method,
                                ImageSlicingMethod::BySize{..}
                            ) => {
                                self.load_options.slicing_method = ImageSlicingMethod::by_size(sprite.width, sprite.height);
                            }
                            ImageSlicingMethodOption::ByNumber if ! matches!(
                                self.load_options.slicing_method,
                                ImageSlicingMethod::ByNumber{..}
                            ) => {
                                self.load_options.slicing_method = ImageSlicingMethod::by_number(1, 1);
                            }
                            _ => {}
                        }

                        match self.load_options.slicing_method {
                            ImageSlicingMethod::BySize { width, height } => {
                                let (mut w, mut h) = (width, height);
                                ui.label("Width:");  ui.add(egui::Slider::new(&mut w, 1..=256)); ui.end_row();
                                ui.label("Height:"); ui.add(egui::Slider::new(&mut h, 1..=256)); ui.end_row();
                                if w != width || h != height {
                                    self.load_options.slicing_method = ImageSlicingMethod::by_size(w, h);
                                }
                            }
                            ImageSlicingMethod::ByNumber { nx, ny } => {
                                let (mut x, mut y) = (nx, ny);
                                ui.label("Num X:"); ui.add(egui::Slider::new(&mut x, 1..=64)); ui.end_row();
                                ui.label("Num Y:"); ui.add(egui::Slider::new(&mut y, 1..=64)); ui.end_row();
                                if x != nx || y != ny {
                                    self.load_options.slicing_method = ImageSlicingMethod::by_number(x, y);
                                }
                            }
                        }

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
                        self.import_sprite_sys_dlg_id.clone(),
                        "sprite",
                        "Import Sprite",
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
