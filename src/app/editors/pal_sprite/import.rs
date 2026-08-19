use crate::image::{
    ImageCollectionIO,
    ImageSlicingMethod,
    ImageLoadOptions,
};
use crate::data_asset::{
    PalSprite,
    PalSpriteDepth,
};

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
    ImageSlicingMethodOption,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportPaletteOption {
    KeepCurrent,
    GenerateNew,
}

impl ImportPaletteOption {
    fn text(&self) -> &str {
        match self {
            ImportPaletteOption::KeepCurrent => "keep current",
            ImportPaletteOption::GenerateNew => "generate new",
        }
    }
}

pub struct ImportDialog {
    pub open: bool,
    dlg_window_id: egui::Id,
    load_options: ImageLoadOptions,
    import_palette: ImportPaletteOption,
    palette_depth: PalSpriteDepth,
    import_sys_dlg_id: String,
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
            dlg_window_id: egui::Id::new("dlg_pal_sprite_import"),
            load_options: Self::DEFAULT_LOAD_OPTIONS,
            import_palette: ImportPaletteOption::GenerateNew,
            palette_depth: PalSpriteDepth::Bpp4,
            import_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pal_sprite: &PalSprite) {
        self.load_options =  Self::DEFAULT_LOAD_OPTIONS;
        self.import_palette = ImportPaletteOption::GenerateNew;
        self.palette_depth = PalSpriteDepth::Bpp4;
        self.import_sys_dlg_id.replace_range(.., &format!("editor_{}_import_pal_sprite", pal_sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn generate_palette(pal_sprite: &mut PalSprite, depth: PalSpriteDepth) {
        // count how many times each color appears in the image
        let mut color_histogram = vec![0u32; 256];
        for color in pal_sprite.data.iter() {
            let color = *color as usize;
            color_histogram[color] = color_histogram[color].saturating_add(1);
        }

        // sort the most used colors first
        let mut colors = color_histogram.into_iter().enumerate().collect::<Vec<(usize, u32)>>();
        colors.sort_by_key(|(_, num)| -(*num as i64));

        // pick however many colors we need
        let num_colors = depth.num_colors() as usize;
        let colors = colors.into_iter().filter(|(_, n)| *n != 0).take(num_colors).map(|(c, _)| (c & 0xff) as u8).collect::<Vec<u8>>();

        // copy the palette to the sprite and re-calculate color-to-index map
        let num_colors = num_colors.min(colors.len());
        pal_sprite.palette[0..num_colors].copy_from_slice(&colors);
        if num_colors < pal_sprite.palette.len() {
            for color in pal_sprite.palette[num_colors..].iter_mut() {
                *color = 0;
            }
        }
        pal_sprite.depth = depth;
        pal_sprite.recalculate_color_to_palette_index_map();

        // force whole image to the new palette
        pal_sprite.force_palette();
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        match file.read_data().and_then(|data| pal_sprite.load_image_png(&data, &self.load_options)) {
            Ok(()) => {
                match self.import_palette {
                    ImportPaletteOption::KeepCurrent => {
                        pal_sprite.force_palette();
                    }
                    ImportPaletteOption::GenerateNew => {
                        Self::generate_palette(pal_sprite, self.palette_depth);
                    }
                }
                true
            }
            Err(e) => {
                wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error importing sprite",
                    "Error importing sprite file.\n\nConsult the log window for more information."
                );
                false
            }
        }
    }

    fn visibility_ui_builder(visible: bool) -> egui::UiBuilder {
        if visible {
            egui::UiBuilder::new()
        } else {
            egui::UiBuilder::new().invisible()
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sys_dlg_id) &&
            self.confirm(file, wc, pal_sprite) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 450.0, "Import Paletted Sprite", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", pal_sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Slice image:");
                        let mut slicing_option = ImageSlicingMethodOption::from_slicing_method(&self.load_options.slicing_method);
                        egui::ComboBox::from_id_salt(format!("editor_{}_import_combo_slicing", pal_sprite.asset.id))
                            .selected_text(slicing_option.text())
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut slicing_option,
                                    ImageSlicingMethodOption::BySize,
                                    ImageSlicingMethodOption::BySize.text()
                                );
                                ui.selectable_value(
                                    &mut slicing_option,
                                    ImageSlicingMethodOption::ByNumber,
                                    ImageSlicingMethodOption::ByNumber.text()
                                );
                            });
                        ui.end_row();
                        match slicing_option {
                            ImageSlicingMethodOption::BySize if ! matches!(
                                self.load_options.slicing_method,
                                ImageSlicingMethod::BySize{..}
                            ) => {
                                self.load_options.slicing_method = ImageSlicingMethod::by_size(pal_sprite.width, pal_sprite.height);
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

                        ui.label("Palette:");
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt(format!("editor_{}_import_combo_palette", pal_sprite.asset.id))
                                .selected_text(self.import_palette.text())
                                .width(50.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.import_palette,
                                        ImportPaletteOption::GenerateNew,
                                        ImportPaletteOption::GenerateNew.text()
                                    );
                                    ui.selectable_value(
                                        &mut self.import_palette,
                                        ImportPaletteOption::KeepCurrent,
                                        ImportPaletteOption::KeepCurrent.text()
                                    );
                                });
                            ui.scope_builder(Self::visibility_ui_builder(self.import_palette == ImportPaletteOption::GenerateNew), |ui| {
                                egui::ComboBox::from_id_salt(format!("editor_{}_import_depth_combo", pal_sprite.asset.id))
                                    .selected_text(format!("{} bpp ({} colors)", self.palette_depth.bits_per_pixel(),
                                                           self.palette_depth.num_colors()))
                                    .width(50.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.palette_depth, PalSpriteDepth::Bpp1, "1 bpp (2 colors)");
                                        ui.selectable_value(&mut self.palette_depth, PalSpriteDepth::Bpp2, "2 bpp (4 colors)");
                                        ui.selectable_value(&mut self.palette_depth, PalSpriteDepth::Bpp4, "4 bpp (16 colors)");
                                    });
                            });
                        });
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Open File").clicked() {
                    wc.sys_dialogs.open_file(
                        Some(wc.egui.window),
                        self.import_sys_dlg_id.clone(),
                        "pal_sprite",
                        "Import Paletted Sprite",
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
