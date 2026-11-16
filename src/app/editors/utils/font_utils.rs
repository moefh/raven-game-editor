use crate::image::{
    ImageCollection,
};
use crate::data_store::{
    Font,
    PropFont,
};

use super::{
    WindowContext,
};

pub trait FontPainter {
    fn font_height(&self) -> f32;
    fn measure(&self, height: f32, text: &str) -> f32;
    fn paint_char(&self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        ch: char,
        pos: egui::Pos2,
        height: f32,
        color: egui::Color32
    ) -> f32;

    fn paint_string(&self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        string: &str,
        pos: egui::Pos2,
        height: f32,
        color: egui::Color32
    ) -> f32 {
        let mut x = 0.0;
        for ch in string.chars() {
            let pos = egui::Pos2::new(pos.x + x, pos.y);
            x += self.paint_char(ui, wc, ch, pos, height, color);
        }
        x
    }
}

pub fn copy_font_chars_to_prop_font(
    font: &Font,
    prop_font: &mut PropFont,
) {
    prop_font.resize(font.height * 2, font.height, PropFont::NUM_CHARS, PropFont::BG_COLOR);

    let height = font.height as usize;
    let font_width = font.width as usize;
    let prop_font_max_width = prop_font.max_width as usize;
    for char_num in 0..Font::NUM_CHARS.min(PropFont::NUM_CHARS) as usize {
        let mut has_space = true;
        for y in 0..height {
            let src_start = (char_num * height + y) * font_width;
            let dst_start = (char_num * height + y) * prop_font_max_width;
            prop_font.data[dst_start..dst_start+font_width].copy_from_slice(
                &font.data[src_start..src_start+font_width]
            );
            if font.data.get(src_start+font_width-1).is_some_and(|&color| color != Font::BG_COLOR) {
                has_space = false;
            }
        }
        if let Some(char_width) = prop_font.char_widths.get_mut(char_num) {
            *char_width = ((font.width & 0xff) as u8).saturating_sub(if has_space { 1 } else { 0 });
        }
    }
}
