use std::path::PathBuf;

use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollectionIO, ImagePixelsCollection};
use crate::data_asset::PropFont;

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
        egui::Id::new("dlg_pfont_import")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pfont: &PropFont) {
        self.filename = None;
        self.display_filename = None;
        self.width = pfont.height;
        self.height = pfont.height;
        self.border = 0;
        self.space_between = 0;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn calc_prop_font_char_width(image: &ImagePixelsCollection, ch: u32) -> u8 {
        if image.width == 0 { return 0; }
        if image.width > 255 { return 255; }

        let ch = ch as usize;
        let image_bg = image.data[0];
        let width = image.width as usize;
        let height = image.height as usize;

        let char_len = width * height;
        let char_off = ch * char_len;
        let char_data = &image.data[char_off .. char_off+char_len];
        let get_line_width = |y| {
            let line = &char_data[y * width .. (y+1) * width];
            for x in (1..width-1).rev() {
                if  line[x] != image_bg {
                    return (x+1) as u8;
                }
            }
            1
        };

        (0..height).map(get_line_width).max().unwrap_or(1)
    }

    fn copy_prop_font_char(pfont: &mut PropFont, image: &ImagePixelsCollection, ch: u32) {
        let ch = ch as usize;
        let pfont_width = pfont.max_width as usize;
        let char_width = pfont.char_widths[ch] as usize;
        let img_width = image.width as usize;
        let height = image.height as usize;
        let image_bg = image.data[0];

        let pfont_char_len = pfont_width * height;
        let pfont_char_off = ch * pfont_char_len;
        let pfont_char_data = &mut pfont.data[pfont_char_off .. pfont_char_off+pfont_char_len];
        let img_char_len = img_width * height;
        let img_char_off = ch * img_char_len;
        let img_char_data = &image.data[img_char_off .. img_char_off+img_char_len];
        for y in 0..height {
            let src_line = &img_char_data[y * img_width .. (y+1) * img_width];
            let dst_line = &mut pfont_char_data[y * pfont_width .. (y+1) * pfont_width];
            for x in 0..char_width {
                dst_line[x] = if src_line[x] == image_bg { PropFont::BG_COLOR } else { PropFont::FG_COLOR };
            }
        }
    }

    fn load_prop_font_from_image(pfont: &mut PropFont, image: &ImagePixelsCollection) {
        pfont.max_width = 2 * image.height;
        pfont.height = image.height;
        pfont.data.resize((pfont.max_width * pfont.height * PropFont::NUM_CHARS) as usize, PropFont::BG_COLOR);
        pfont.data[..].fill(PropFont::BG_COLOR);

        for ch in 0..PropFont::NUM_CHARS {
            pfont.char_widths[ch as usize] = Self::calc_prop_font_char_width(image, ch);
            Self::copy_prop_font_char(pfont, image, ch);
        }
    }

    fn confirm(&mut self, wc: &mut WindowContext, pfont: &mut PropFont) -> bool {
        if let Some(filename) = &self.filename {
            let mut image = ImagePixelsCollection::new(1, 1, 1);
            match image.load_image_png(filename, self.width, self.height, self.border, self.space_between) {
                Ok(num_chars) => {
                    if num_chars == PropFont::NUM_CHARS {
                        image.width = self.width;
                        image.height = self.height;
                        Self::load_prop_font_from_image(pfont, &image);
                        true
                    } else {
                        wc.open_message_box(
                            "Error importing font",
                            format!("Invalid font image: found {} characters, required {}.", num_chars, PropFont::NUM_CHARS),
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

    pub fn show(&mut self, wc: &mut WindowContext, pfont: &mut PropFont) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(format!("editor_{}_import_pfont", pfont.asset.id)) {
            self.display_filename = Some(filename.as_path().file_name().map(|f| f.display().to_string()).unwrap_or("?".to_owned()));
            self.filename = Some(filename);
        }

        let mut confirmed = false;
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Import PropFont");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_import_grid", pfont.asset.id))
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
                                        format!("editor_{}_import_pfont", pfont.asset.id),
                                        "Import PropFont",
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
                    if ui.button("Ok").clicked() && self.confirm(wc, pfont) {
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
