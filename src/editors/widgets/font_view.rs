use egui::{Vec2, Pos2, Rect};

use crate::app::WindowContext;

pub trait FontPainter {
    fn measure(&self, height: f32, text: &str) -> f32;
    fn paint_char(&self, ui: &mut egui::Ui, wc: &mut WindowContext, ch: char, pos: Pos2, height: f32) -> f32;
}

pub struct FontViewWidget {
    pub text: String,
}

impl FontViewWidget {
    const BORDER: f32 = 2.0;
    const SCROLLBAR_HEIGHT: f32 = 10.0;

    pub fn new() -> Self {
        FontViewWidget {
            text: String::from("Hello, world!"),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, wc: &mut WindowContext, font_painter: &impl FontPainter) {
        let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: true, mouse_wheel: true };
        let height = 30.0;
        let width = font_painter.measure(height - 2.0 * Self::BORDER, &self.text);
        egui::ScrollArea::horizontal()
            .auto_shrink([false, true])
            .scroll_source(source)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                let min_size = Vec2::new(ui.available_width().max(width + 2.0 * Self::BORDER), height + Self::SCROLLBAR_HEIGHT);
                let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
                let canvas_rect = Rect {
                    min: response.rect.min,
                    max: Pos2::new(response.rect.max.x, response.rect.min.y + height),
                };

                ui.shrink_clip_rect(canvas_rect);
                painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);

                let mut x = Self::BORDER;
                for ch in self.text.chars() {
                    x += font_painter.paint_char(ui, wc, ch, canvas_rect.min + Vec2::new(x, Self::BORDER), height - 2.0 * Self::BORDER);
                }
            });
    }
}
