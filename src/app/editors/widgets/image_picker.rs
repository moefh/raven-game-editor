use egui::{
    Vec2,
    Pos2,
    Color32,
    Rect,
};

use crate::image::ImageCollection;
use super::image_editor::ImageDisplay;

use super::super::super::AppSettings;

pub struct ImagePickerWidget {
    pub allow_empty_selection: bool,
    pub allow_second_selection: bool,
    pub zoom: f32,
    pub display: ImageDisplay,
    selected_image: Option<u32>,
    selected_image_right: Option<u32>,
    selected_image_changed: bool,
    selected_image_right_changed: bool,
}

impl ImagePickerWidget {
    const BORDER: f32 = 3.0;

    pub fn new() -> Self {
        ImagePickerWidget {
            allow_empty_selection: false,
            allow_second_selection: false,
            zoom: 1.0,
            selected_image: Some(0),
            selected_image_right: None,
            selected_image_changed: false,
            selected_image_right_changed: false,
            display: ImageDisplay::new(0),
        }
    }

    pub fn use_as_palette(mut self, use_as_palette: bool) -> Self {
        self.allow_second_selection = use_as_palette;
        self.allow_empty_selection = use_as_palette;
        self
    }

    pub fn get_selected_image(&self) -> Option<u32> {
        self.selected_image
    }

    pub fn get_selected_image_right(&self) -> Option<u32> {
        self.selected_image_right
    }

    pub fn set_selected_image(&mut self, selected_image: Option<u32>) {
        self.selected_image = selected_image;
        self.selected_image_changed = true;
    }

    pub fn set_selected_image_right(&mut self, selected_image: Option<u32>) {
        self.selected_image_right = selected_image;
        self.selected_image_right_changed = true;
    }

    fn ui_pos_to_selection(&self, ui_pos: f32) -> Option<u32> {
        if self.allow_empty_selection {
            if ui_pos == 0.0 { None } else { Some((ui_pos - 1.0).floor() as u32) }
        } else {
            Some(ui_pos.floor().max(0.0) as u32)
        }
    }

    fn selection_to_ui_pos(&self, selection: Option<u32>) -> f32 {
        if let Some(index) = selection {
            index as f32 + if self.allow_empty_selection { 1.0 } else { 0.0 }
        } else {
            0.0
        }
    }

    fn draw_selection_rectangle(
        &self,
        painter: &egui::Painter,
        canvas_pos: Pos2,
        image_size: Vec2,
        selected_image: Option<u32>,
        shrink: f32,
        colors: (Color32, Color32)
    ) {
        let pos = canvas_pos + Vec2::new(shrink, self.selection_to_ui_pos(selected_image) * image_size.y + shrink);
        let sel_rect = Rect::from_min_size(pos, image_size - Vec2::splat(2.0 * shrink) + Vec2::splat(2.0 * Self::BORDER));
        let stroke = egui::Stroke::new(3.0, colors.0);
        painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

        let in_stroke = egui::Stroke::new(1.0, colors.1);
        let sel_in_rect = sel_rect.expand2(Vec2::splat(-2.0));
        painter.rect_stroke(sel_in_rect, egui::CornerRadius::ZERO, in_stroke, egui::StrokeKind::Inside);
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _settings: &AppSettings,
        image: &impl ImageCollection,
        texture: &egui::TextureHandle,
        bg_color: Color32
    ) {
        let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: egui::scroll_area::DragScroll::Never, mouse_wheel: true };
        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source);
        let image_size = self.zoom * image.get_item_size();

        // scroll to selected image if changed
        if let Some(scroll_to_pos) = if self.selected_image_changed {
            self.selected_image_changed = false;
            Some(self.selection_to_ui_pos(self.selected_image))
        } else if self.selected_image_right_changed {
            self.selected_image_right_changed = false;
            Some(self.selection_to_ui_pos(self.selected_image_right))
        } else {
            None
        } {
            scroll_area = scroll_area.scroll_offset(Vec2::new(0.0, scroll_to_pos * image_size.y));
        }

        let resp = scroll_area.show(ui, |ui| {
            let empty_item_size = self.zoom * if self.allow_empty_selection { Vec2::new(0.0, image.height() as f32) } else { Vec2::ZERO };
            let image_picker_size = self.zoom * image.get_full_size() + empty_item_size + 2.0 * Vec2::splat(Self::BORDER);
            let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(16.0, 6.0)).min(Vec2::new(200.0, f32::INFINITY));
            let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
            let space = response.rect;
            let images_rect = Rect::from_min_size(space.min + empty_item_size + Vec2::splat(Self::BORDER), self.zoom * image.get_full_size());
            let canvas_rect = Rect::from_min_size(space.min, image_picker_size);

            // draw background
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::BLACK);
            if self.allow_empty_selection {
                let empty_image_rect = Rect::from_min_size(space.min + Vec2::splat(Self::BORDER), self.zoom * image.get_item_size());
                painter.rect_filled(empty_image_rect, egui::CornerRadius::ZERO, bg_color);
            }
            painter.rect_filled(images_rect, egui::CornerRadius::ZERO, bg_color);

            // draw items
            egui::Image::from_texture((texture.id(), image_picker_size)).uv(super::FULL_UV).paint_at(ui, images_rect);

            // draw selection rectangles
            if self.allow_second_selection {
                self.draw_selection_rectangle(&painter, canvas_rect.min, image_size, self.selected_image_right,
                                              3.0, (Color32::RED, Color32::WHITE));
            }
            self.draw_selection_rectangle(&painter, canvas_rect.min, image_size, self.selected_image,
                                          0.0, (Color32::BLUE, Color32::WHITE));

            response
        });
        if let Some(pointer_pos) = resp.inner.interact_pointer_pos() {
            let pos = pointer_pos - resp.inner_rect.min + resp.state.offset;
            if pos.x >= 0.0 && pos.x <= resp.inner_rect.width() {
                let frame_size = self.zoom * image.get_item_size();
                let num_items = image.num_items() as i32 + if self.allow_empty_selection { 1 } else { 0 };
                let selection = self.ui_pos_to_selection(f32::min((pos.y / frame_size.y).floor(), (num_items - 1) as f32));
                if resp.inner.dragged_by(egui::PointerButton::Primary) {
                    self.selected_image = selection;
                } else if resp.inner.dragged_by(egui::PointerButton::Secondary) {
                    self.selected_image_right = selection;
                }
            }
        };
    }
}
