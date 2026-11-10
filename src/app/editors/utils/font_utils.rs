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
