use egui::{
    Vec2,
    Sense,
    Rect,
    Image,
};

use crate::image::{
    ImageCollection,
};

use super::image_editor::ImageDisplay;

use super::super::{
    WindowContext,
};

pub struct ImageOrganizerWidget {
    pub num_images_x: i32,
    pub num_images_y: i32,
    pub display: ImageDisplay,
    pub zoom: f32,
    pub indices: Vec<u32>,
    sel_slot: Option<i32>,
}

impl ImageOrganizerWidget {
    pub fn new() -> Self{
        ImageOrganizerWidget {
            num_images_x: 16,
            num_images_y: 8,
            display: ImageDisplay::new(ImageDisplay::GRID),
            zoom: 3.0,
            indices: Vec::new(),
            sel_slot: None,
        }
    }

    pub fn reset(&mut self, num_images: u32) {
        self.indices.resize(num_images as usize, 0);
        for (i, index) in self.indices.iter_mut().enumerate() {
            *index = i as u32;
        }
    }

    fn draw_images(
        &self,
        painter: &egui::Painter,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        rect: Rect,
        image: &impl ImageCollection,
    ) {
        // images
        let unzoomed_image_size = Vec2::new(image.width() as f32, image.height() as f32);
        let image_size = Vec2::new(rect.width() / (self.num_images_x as f32), rect.height() / (self.num_images_y as f32));
        for y in 0..self.num_images_y {
            for x in 0..self.num_images_x {
                let image_rect = Rect::from_min_size(rect.min + image_size * Vec2::new(x as f32, y as f32), image_size);
                let slot_index = (y * self.num_images_x + x).unsigned_abs() as usize;
                if let Some(image_index) = self.indices.get(slot_index) {
                    let uv = image.get_item_uv(*image_index);
                    let slot = image.texture_slot(self.display.is_transparent(), false);
                    let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);
                    Image::from_texture((texture.id(), unzoomed_image_size)).uv(uv).paint_at(ui, image_rect);
                }
            }
        }

        // grid
        if self.display.has_bits(ImageDisplay::GRID) {
            let num_slots_empty = ((self.num_images_x * self.num_images_y) - image.num_items() as i32).max(0);
            let stroke = egui::Stroke::new(1.0, wc.settings.map_grid_color);
            for y in 0..self.num_images_y+1 {
                let cy = rect.min.y + y as f32 * image_size.y;
                if y < self.num_images_y || num_slots_empty == 0 {
                    painter.hline(rect.x_range(), cy, stroke);
                } else {
                    let range = rect.min.x ..= rect.max.x - num_slots_empty as f32 * image_size.x;
                    painter.hline(range, cy, stroke);
                }
            }
            for x in 0..self.num_images_x+1 {
                let cx = rect.min.x + x as f32 * image_size.x;
                if x <= self.num_images_x - num_slots_empty {
                    painter.vline(cx, rect.y_range(), stroke);
                } else {
                    let range = rect.min.y ..= rect.max.y - image_size.y;
                    painter.vline(cx, range, stroke);
                }
            }
        }

        // selection rectangle
        if let Some(slot) = self.sel_slot {
            let x = slot % self.num_images_x;
            let y = slot / self.num_images_x;
            let image_rect = Rect::from_min_size(rect.min + image_size * Vec2::new(x as f32, y as f32), image_size);
            let stroke = egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 255));
            painter.rect_stroke(image_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
        }
    }

    fn move_slot(&mut self, from: i32, to: i32) {
        if from == to || from < 0 || to < 0 || from >= self.indices.len() as i32 || to >= self.indices.len() as i32 {
            return;
        }
        let from = from as usize;
        let to = to as usize;
        let saved = self.indices[from];
        if from < to {
            self.indices.copy_within(from+1..to+1, from);
        } else {
            self.indices.copy_within(to..from, to+1);
        }
        self.indices[to] = saved;
    }

    fn handle_mouse(&mut self, response: &egui::Response, image: &impl ImageCollection) {
        if response.drag_stopped() {
            self.sel_slot = None;
            return;
        }

        if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
            let image_size = Vec2::new(response.rect.width() / (self.num_images_x as f32), response.rect.height() / (self.num_images_y as f32));
            let x = ((pos.x - response.rect.min.x) / image_size.x).floor() as i32;
            let y = ((pos.y - response.rect.min.y) / image_size.y).floor() as i32;
            let slot = y * self.num_images_x + x;
            if slot >= 0 && slot < image.num_items() as i32 {
                if let Some(old_slot) = self.sel_slot && old_slot != slot {
                    self.move_slot(old_slot, slot);
                }
                self.sel_slot = Some(slot);
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, image: &impl ImageCollection) {
        let min_size = self.zoom * Vec2::new(
            image.width() as f32 * self.num_images_x as f32,
            image.height() as f32 * self.num_images_y as f32
        ).min(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        self.draw_images(&painter, ui, wc, response.rect, image);
        self.handle_mouse(&response, image);
    }
}
