use egui::{Vec2, Sense, Image, Rect, Pos2};

use crate::data_asset::SpriteAnimationFrame;
use crate::image::ImageCollection;

pub struct SpriteFrameListView<'a> {
    frame_indices: &'a [SpriteAnimationFrame],
    foot_overlap: i8,
    selected_index: usize,
}

impl<'a> SpriteFrameListView<'a> {
    pub fn new(frame_indices: &'a [SpriteAnimationFrame], foot_overlap: i8, selected_index: usize) -> Self {
        SpriteFrameListView {
            frame_indices,
            foot_overlap,
            selected_index,
        }
    }

    fn get_frame_rect(index: usize, y_offset: f32, image_size: Vec2, canvas_rect: Rect) -> Rect {
        Rect {
            min: Pos2 {
                x: canvas_rect.min.x + index as f32 * image_size.x,
                y: canvas_rect.min.y + y_offset,
            },
            max: Pos2 {
                x: canvas_rect.min.x + (index+1) as f32 * image_size.x,
                y: canvas_rect.min.y + y_offset + image_size.y,
            },
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, image: &impl ImageCollection, texture: &egui::TextureHandle)
                -> egui::scroll_area::ScrollAreaOutput<egui::Response> {
        let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true };
        egui::ScrollArea::horizontal().auto_shrink([false, false]).scroll_source(source).show(ui, |ui| {
            let use_foot_frames = self.frame_indices.iter().any(|f| f.foot_index.is_some());
            let foot_overlap = if use_foot_frames { self.foot_overlap as f32 } else { 0.0 };
            let image_size = image.get_item_size();
            let image_cell_size = Vec2::new(image_size.x, image_size.y * if use_foot_frames { 2.0 } else { 1.0 } - foot_overlap);
            let image_picker_size = Vec2::new(image_cell_size.x * self.frame_indices.len() as f32, image_cell_size.y);
            let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(0.0, 10.0)).max(Vec2::new(ui.available_width(), 0.0));
            let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
            let space = response.rect;
            let canvas_rect = Rect {
                min: space.min,
                max: space.min + image_picker_size,
            };

            // draw background
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0xe0u8, 0xffu8, 0xffu8));
            let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
            painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

            // draw items
            for (index, frame) in self.frame_indices.iter().enumerate() {
                if let Some(head_index) = frame.head_index {
                    let uv = image.get_item_uv(head_index as u32);
                    let rect = Self::get_frame_rect(index, 0.0, image_size, canvas_rect);
                    Image::from_texture((texture.id(), image_picker_size)).uv(uv).paint_at(ui, rect);
                }
                if use_foot_frames && let Some(foot_index) = frame.foot_index {
                    let uv = image.get_item_uv(foot_index as u32);
                    let rect = Self::get_frame_rect(index, image_size.y - self.foot_overlap as f32, image_size, canvas_rect);
                    Image::from_texture((texture.id(), image_picker_size)).uv(uv).paint_at(ui, rect);
                }
            }

            // draw selection rectangle
            if self.selected_index < self.frame_indices.len() {
                let stroke = egui::Stroke::new(3.0, egui::Color32::BLACK);
                let sel_rect = Self::get_frame_rect(self.selected_index, 0.0, image_size, canvas_rect);
                painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

                let in_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                let sel_in_rect = sel_rect.expand2(Vec2::splat(-2.0));
                painter.rect_stroke(sel_in_rect, egui::CornerRadius::ZERO, in_stroke, egui::StrokeKind::Inside);
            }

            response
        })
    }
}
