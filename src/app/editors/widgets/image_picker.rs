use std::collections::HashMap;

use egui::{
    Vec2,
    Pos2,
    Color32,
    Rect,
};

use crate::image::ImageCollection;
use super::image_editor::ImageDisplay;

use super::super::WindowContext;

pub struct ImagePickerWidget {
    pub allow_empty_selection: bool,
    pub allow_r_selection: bool,
    pub selection_set: usize,
    pub zoom: Option<f32>,
    pub display: ImageDisplay,
    pub image_l_picked: bool,
    pub image_r_picked: bool,
    pub custom_bg_color: Option<egui::Color32>,
    selected_image_l: HashMap<usize, Option<u32>>,
    selected_image_r: HashMap<usize, Option<u32>>,
    selected_image_l_changed: bool,
    selected_image_r_changed: bool,
}

impl ImagePickerWidget {
    const BORDER: f32 = 3.0;

    pub fn new() -> Self {
        ImagePickerWidget {
            allow_empty_selection: false,
            allow_r_selection: false,
            zoom: None,
            image_l_picked: false,
            image_r_picked: true,
            selection_set: 0,
            selected_image_l: HashMap::from([(0, Some(0)), (1, Some(0)), (1, Some(0))]),
            selected_image_r: HashMap::new(),
            selected_image_l_changed: false,
            selected_image_r_changed: false,
            display: ImageDisplay::new(0),
            custom_bg_color: None,
        }
    }

    pub fn use_as_palette(mut self, use_as_palette: bool) -> Self {
        self.allow_r_selection = use_as_palette;
        self.allow_empty_selection = use_as_palette;
        self
    }

    pub fn get_selected_image_l(&self) -> Option<u32> {
        self.selected_image_l.get(&self.selection_set).copied().unwrap_or(None)
    }

    pub fn get_selected_image_r(&self) -> Option<u32> {
        self.selected_image_r.get(&self.selection_set).copied().unwrap_or(None)
    }

    pub fn set_selected_image_l(&mut self, selected_image: Option<u32>) {
        self.selected_image_l.insert(self.selection_set, selected_image);
        self.selected_image_l_changed = true;
    }

    pub fn set_selected_image_r(&mut self, selected_image: Option<u32>) {
        self.selected_image_r.insert(self.selection_set, selected_image);
        self.selected_image_r_changed = true;
    }

    pub fn get_selected_image_l_for_set(&self, selection_set: usize) -> Option<u32> {
        self.selected_image_l.get(&selection_set).copied().unwrap_or(None)
    }

    pub fn get_selected_image_r_for_set(&self, selection_set: usize) -> Option<u32> {
        self.selected_image_r.get(&selection_set).copied().unwrap_or(None)
    }

    pub fn force_selection_into_visibility(&mut self) {
        self.selected_image_l_changed = true;
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
        wc: &mut WindowContext,
        image: &impl ImageCollection,
    ) {
        let bg_color = self.custom_bg_color.unwrap_or(wc.settings.image_bg_color);
        let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: egui::scroll_area::DragScroll::Never, mouse_wheel: true };
        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source);
        let zoom = self.zoom.unwrap_or(wc.settings.tile_picker_zoom as f32 / 100.0);
        let image_size = zoom * image.get_item_size();

        // scroll to selected image if changed
        if let Some(scroll_to_pos) = if self.selected_image_l_changed {
            self.selected_image_l_changed = false;
            Some(self.selection_to_ui_pos(self.get_selected_image_l()))
        } else if self.selected_image_r_changed {
            self.selected_image_r_changed = false;
            Some(self.selection_to_ui_pos(self.get_selected_image_r()))
        } else {
            None
        } {
            scroll_area = scroll_area.scroll_offset(Vec2::new(0.0, scroll_to_pos * image_size.y));
        }

        let resp = scroll_area.show(ui, |ui| {
            let empty_item_size = zoom * if self.allow_empty_selection { Vec2::new(0.0, image.height() as f32) } else { Vec2::ZERO };
            let image_picker_size = zoom * image.get_full_size() + empty_item_size + 2.0 * Vec2::splat(Self::BORDER);
            let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(16.0, 6.0)).min(Vec2::new(200.0, f32::INFINITY));
            let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
            let space = response.rect;
            let images_rect = Rect::from_min_size(space.min + empty_item_size + Vec2::splat(Self::BORDER), zoom * image.get_full_size());
            let canvas_rect = Rect::from_min_size(space.min, image_picker_size);

            // draw background
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::BLACK);
            if self.allow_empty_selection {
                let empty_image_rect = Rect::from_min_size(space.min + Vec2::splat(Self::BORDER), zoom * image.get_item_size());
                painter.rect_filled(empty_image_rect, egui::CornerRadius::ZERO, bg_color);
            }
            painter.rect_filled(images_rect, egui::CornerRadius::ZERO, bg_color);

            // draw items
            let slot = image.texture_slot(self.display.is_transparent(), false);
            let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);
            egui::Image::from_texture((texture.id(), image_picker_size)).uv(super::FULL_UV).paint_at(ui, images_rect);

            // draw selection rectangles
            if self.allow_r_selection {
                self.draw_selection_rectangle(
                    &painter,
                    canvas_rect.min,
                    image_size,
                    self.get_selected_image_r(),
                    3.0,
                    (Color32::RED, Color32::WHITE)
                );
            }
            self.draw_selection_rectangle(
                &painter,
                canvas_rect.min,
                image_size,
                self.get_selected_image_l(),
                0.0,
                (Color32::BLUE, Color32::WHITE)
            );

            response
        });
        self.image_l_picked = false;
        self.image_r_picked = false;
        if let Some(pointer_pos) = resp.inner.interact_pointer_pos() {
            let pos = pointer_pos - resp.inner_rect.min + resp.state.offset;
            if pos.x >= 0.0 && pos.x <= resp.inner_rect.width() {
                let frame_size = zoom * image.get_item_size();
                let num_items = image.num_items() as i32 + if self.allow_empty_selection { 1 } else { 0 };
                let selection = self.ui_pos_to_selection(f32::min((pos.y / frame_size.y).floor(), (num_items - 1) as f32));
                if resp.inner.dragged_by(egui::PointerButton::Primary) {
                    self.selected_image_l.insert(self.selection_set, selection);
                    self.image_l_picked = true;
                } else if resp.inner.dragged_by(egui::PointerButton::Secondary) {
                    self.selected_image_r.insert(self.selection_set, selection);
                    self.image_r_picked = true;
                }
            }
        }
    }
}
