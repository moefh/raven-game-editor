use egui::{
    Vec2,
    Sense,
    Image,
    Rect,
};

use crate::data_asset::SpriteAnimationFrame;
use crate::image::ImageCollection;

use super::super::WindowContext;

pub struct SpriteFrameListView {
    pub allow_selection: bool,
    pub track_drag_and_drop: bool,
    pub selected_frame: usize,
    pub hovered_frame: Option<usize>,
    dragging: bool,
}

impl SpriteFrameListView {
    pub fn new(allow_selection: bool, track_drag_and_drop: bool) -> Self {
        SpriteFrameListView {
            allow_selection,
            track_drag_and_drop,
            selected_frame: 0,
            hovered_frame: None,
            dragging: false,
        }
    }

    fn get_image_size(size: Vec2) -> Vec2 {
        let zoom = if size.x > size.y {
            64.0 / size.x
        } else {
            64.0 / size.y
        };
        zoom * size
    }

    fn get_frame_rect(index: usize, y_offset: f32, image_size: Vec2, canvas_rect: Rect) -> Rect {
        Rect::from_min_size(canvas_rect.min + Vec2::new(index as f32 * image_size.x, y_offset), image_size)
    }

    fn get_frame_at(
        pos: egui::Pos2,
        scroll: &egui::scroll_area::ScrollAreaOutput<egui::Response>,
        image_width: f32,
        num_frames: usize
    ) -> Option<usize> {
        if num_frames != 0 {
            let pos_offset = pos - scroll.inner_rect.min + scroll.state.offset;
            let frame = (pos_offset.x / image_width).floor();
            if frame >= 0.0 && (frame as usize) < num_frames {
                return Some(frame as usize);
            }
        }
        None
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        frame_indices: &[SpriteAnimationFrame],
        foot_overlap: i8,
        image: &impl ImageCollection,
        transparent_display: bool,
    ) -> egui::scroll_area::ScrollAreaOutput<egui::Response> {
        let slot = image.texture_slot(transparent_display, false);
        let texture = image.load_texture(wc.tex_man, wc.egui.ctx, slot, false);
        let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: egui::scroll_area::DragScroll::Never, mouse_wheel: true };
        let image_size = Self::get_image_size(image.get_item_size());
        let scroll = egui::ScrollArea::horizontal().auto_shrink([false, false]).scroll_source(source).show(ui, |ui| {
            let use_foot_frames = frame_indices.iter().any(|f| f.foot_index.is_some());
            let foot_overlap = if use_foot_frames { foot_overlap as f32 } else { 0.0 };
            let image_cell_size = Vec2::new(image_size.x, image_size.y * if use_foot_frames { 2.0 } else { 1.0 } - foot_overlap);
            let image_picker_size = Vec2::new(image_cell_size.x * frame_indices.len() as f32, image_cell_size.y);
            let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(0.0, 10.0)).max(Vec2::new(ui.available_width(), 0.0));
            let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
            let space = response.rect;
            let canvas_rect = Rect::from_min_size(space.min, image_picker_size);

            // draw background
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);
            let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
            painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

            // draw items
            for (index, frame) in frame_indices.iter().enumerate() {
                if let Some(head_index) = frame.head_index {
                    let uv = image.get_item_uv(head_index as u32);
                    let rect = Self::get_frame_rect(index, 0.0, image_size, canvas_rect);
                    Image::from_texture((texture.id(), image_picker_size)).uv(uv).paint_at(ui, rect);
                }
                if use_foot_frames && let Some(foot_index) = frame.foot_index {
                    let uv = image.get_item_uv(foot_index as u32);
                    let rect = Self::get_frame_rect(index, image_size.y - foot_overlap, image_size, canvas_rect);
                    Image::from_texture((texture.id(), image_picker_size)).uv(uv).paint_at(ui, rect);
                }
            }

            // draw selection rectangle
            if self.allow_selection && self.selected_frame < frame_indices.len() {
                let stroke = egui::Stroke::new(3.0, egui::Color32::BLACK);
                let sel_rect = Self::get_frame_rect(self.selected_frame, 0.0, image_size, canvas_rect);
                painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

                let in_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                let sel_in_rect = sel_rect.expand2(Vec2::splat(-2.0));
                painter.rect_stroke(sel_in_rect, egui::CornerRadius::ZERO, in_stroke, egui::StrokeKind::Inside);
            }

            response
        });

        // handle clicking/dragging
        if self.dragging && (! self.track_drag_and_drop || ! scroll.inner.dragged()) {
            self.dragging = false;
        }
        if ! self.dragging && let Some(pointer_pos) = scroll.inner.interact_pointer_pos() && scroll.inner_rect.contains(pointer_pos) {
            if self.track_drag_and_drop && scroll.inner.drag_started() {
                self.dragging = true;
            }
            if let Some(frame) = Self::get_frame_at(pointer_pos, &scroll, image_size.x, frame_indices.len()) {
                self.selected_frame = frame;
            }
        }

        // handle hover
        if let Some(hover_pos) = scroll.inner.hover_pos() && scroll.inner_rect.contains(hover_pos) {
            self.hovered_frame = Self::get_frame_at(hover_pos, &scroll, image_size.x, frame_indices.len());
        } else if self.hovered_frame.is_some() {
            self.hovered_frame = None;
        }

        scroll
    }
}
