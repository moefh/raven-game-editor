use crate::image::{
    ImageCollectionIO,
    ImagePixelsCollection,
    ImageSlicingMethod,
    ImageLoadOptions,
};
use crate::data_asset::PropFont;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
};

trait PropFontFromImage {
    fn calc_char_width(image: &ImagePixelsCollection, ch: u32) -> u8;
    fn copy_char(&mut self, image: &ImagePixelsCollection, ch: u32);
    fn load_from_image(&mut self, image: &ImagePixelsCollection);
}

impl PropFontFromImage for PropFont {
    fn calc_char_width(image: &ImagePixelsCollection, ch: u32) -> u8 {
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

    fn copy_char(&mut self, image: &ImagePixelsCollection, ch: u32) {
        let ch = ch as usize;
        let pfont_width = self.max_width as usize;
        let char_width = self.char_widths[ch] as usize;
        let img_width = image.width as usize;
        let height = image.height as usize;
        let image_bg = image.data[0];

        let pfont_char_len = pfont_width * height;
        let pfont_char_off = ch * pfont_char_len;
        let pfont_char_data = &mut self.data[pfont_char_off .. pfont_char_off+pfont_char_len];
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

    fn load_from_image(&mut self, image: &ImagePixelsCollection) {
        self.max_width = 2 * image.height;
        self.height = image.height;
        self.data.resize((self.max_width * self.height * PropFont::NUM_CHARS) as usize, PropFont::BG_COLOR);
        self.data[..].fill(PropFont::BG_COLOR);

        for ch in 0..PropFont::NUM_CHARS {
            self.char_widths[ch as usize] = Self::calc_char_width(image, ch);
            self.copy_char(image, ch);
        }
    }
}

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
            dlg_window_id: egui::Id::new("dlg_pfont_import"),
            import_sys_dlg_id: String::new(),
            width: 0,
            height: 0,
            border: 0,
            space_between: 0,
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pfont: &PropFont) {
        self.width = pfont.height;
        self.height = pfont.height;
        self.border = 0;
        self.space_between = 0;
        self.import_sys_dlg_id.replace_range(.., &format!("editor_{}_import_pfont", pfont.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, pfont: &mut PropFont) -> bool {
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
                if image.num_items == PropFont::NUM_CHARS {
                    pfont.load_from_image(&image);
                    true
                } else {
                    wc.open_message_box(
                        "Error importing font",
                        format!("Invalid font image: found {} characters, required {}.", image.num_items, PropFont::NUM_CHARS),
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

    pub fn show(&mut self, wc: &mut WindowContext, pfont: &mut PropFont) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sys_dlg_id) &&
            self.confirm(file, wc, pfont) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Import Prop Font", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", pfont.asset.id))
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
                        self.import_sys_dlg_id.clone(),
                        "prop_font",
                        "Import PropFont",
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
