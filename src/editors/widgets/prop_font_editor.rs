use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::PropFont;
use crate::app::WindowContext;

use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub struct PropFontEditorWidget {
    pub selected_char: u32,
    pub image_changed: bool,
}

impl PropFontEditorWidget {
    pub fn new() -> Self {
        PropFontEditorWidget {
            selected_char: 0,
            image_changed: false,
        }
    }

    pub fn with_selected_char(mut self, selected_char: u32) -> Self {
        self.selected_char = selected_char;
        self
    }

    pub fn get_selected_char_width(&self, prop_font: &PropFont) -> u32 {
        prop_font.char_widths.get(self.selected_char as usize).map_or(1, |&v| v) as u32
    }

    fn get_click_color(resp: &egui::Response) -> Option<u8> {
        if resp.dragged_by(egui::PointerButton::Primary) {
            Some(PropFont::FG_COLOR)
        } else if resp.dragged_by(egui::PointerButton::Secondary) {
            Some(PropFont::BG_COLOR)
        } else {
            None
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, prop_font: &mut PropFont) {
        let texture = prop_font.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, self.image_changed);
        if self.image_changed { self.image_changed = false; }
        let char_width = self.get_selected_char_width(prop_font);
        let image_size = Vec2::new(char_width as f32, prop_font.height as f32);
        let min_size = Vec2::splat(100.0).min(image_size + Vec2::splat(10.0)).max(ui.available_size());
        let (resp, painter) = ui.allocate_painter(min_size, Sense::drag());

        let resp_size = resp.rect.size();
        let (zoomx, zoomy) = (resp_size.x / (image_size.x + 1.0), (resp_size.y / (image_size.y + 1.0)));
        let image_zoom = f32::max(f32::min(zoomx, zoomy).floor(), 1.0);
        let center = resp.rect.center();
        let canvas_rect = Rect {
            min: center - image_zoom * image_size / 2.0,
            max: center + image_zoom * image_size / 2.0,
        };

        // draw background
        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);

        // draw image
        let item_uv = Rect {
            min: Pos2::new(0.0, self.selected_char as f32 / PropFont::NUM_CHARS as f32),
            max: Pos2::new(char_width as f32 / prop_font.max_width as f32, (self.selected_char+1) as f32 / PropFont::NUM_CHARS as f32),
        };
        Image::from_texture((texture.id(), image_size)).uv(item_uv).paint_at(ui, canvas_rect);

        // draw border
        let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
        painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // draw grid
        let canvas_size = canvas_rect.size();
        let display_grid = f32::min(canvas_size.x, canvas_size.y) / f32::max(image_size.x, image_size.y) > 3.0;
        if display_grid {
            let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(112, 112, 112));
            for y in 0..=prop_font.height {
                let py = canvas_rect.min.y + canvas_rect.height() * y as f32 / prop_font.height as f32;
                painter.hline(canvas_rect.x_range(), py, stroke);
            }
            for x in 0..=char_width {
                let px = canvas_rect.min.x + canvas_rect.width() * x as f32 / char_width as f32;
                painter.vline(px, canvas_rect.y_range(), stroke);
            }
        }
        let canvas_to_image = emath::RectTransform::from_to(
            canvas_rect,
            Rect { min: Pos2::ZERO, max: Pos2::ZERO + image_size }
        );

        if let Some(pointer_pos) = resp.interact_pointer_pos() && canvas_rect.contains(pointer_pos) {
            let image_pos = canvas_to_image * pointer_pos;
            let x = image_pos.x as i32;
            let y = image_pos.y as i32;
            if let Some(color) = Self::get_click_color(&resp) && prop_font.set_pixel(x, y, self.selected_char, color) {
                self.image_changed = true;
            }
        }
    }
}
