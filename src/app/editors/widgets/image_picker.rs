use std::collections::HashMap;

use egui::{
    Vec2,
    Pos2,
    Color32,
    Rect,
};

use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::misc::{
    STATIC_IMAGES,
};

use super::image_editor::{
    ImageDisplay,
};
use super::super::{
    get_setting_zoom,
    WindowContext,
};

#[derive(Clone, Copy)]
pub enum ImagePickerSelection {
    Empty(u32),
    None,
    Some(u32),
}

impl ImagePickerSelection {
    pub fn unwrap_or(self, def: u32) -> u32 {
        if let ImagePickerSelection::Some(val) = self {
            val
        } else {
            def
        }
    }
}

pub struct ImagePickerWidget {
    pub num_empty_selections: u32,
    pub allow_r_selection: bool,
    pub selection_set: usize,
    pub zoom: Option<f32>,
    pub display: ImageDisplay,
    pub image_l_picked: bool,
    pub image_r_picked: bool,
    pub custom_bg_color: Option<egui::Color32>,
    selected_image_l: HashMap<usize, ImagePickerSelection>,
    selected_image_r: HashMap<usize, ImagePickerSelection>,
    selected_image_l_changed: bool,
    selected_image_r_changed: bool,
}

impl ImagePickerWidget {
    const BORDER: f32 = 3.0;

    pub fn new() -> Self {
        ImagePickerWidget {
            num_empty_selections: 0,
            allow_r_selection: false,
            zoom: None,
            image_l_picked: false,
            image_r_picked: true,
            selection_set: 0,
            selected_image_l: HashMap::from([
                (0, ImagePickerSelection::Some(0)),
                (1, ImagePickerSelection::Some(0)),
                (2, ImagePickerSelection::Some(0)),
            ]),
            selected_image_r: HashMap::new(),
            selected_image_l_changed: false,
            selected_image_r_changed: false,
            display: ImageDisplay::new(0),
            custom_bg_color: None,
        }
    }

    pub fn use_as_palette(mut self, use_as_palette: bool) -> Self {
        self.allow_r_selection = use_as_palette;
        self.num_empty_selections = if use_as_palette { 1 } else { 0 };
        self
    }

    pub fn with_empty_selection(mut self) -> Self {
        self.num_empty_selections = 1;
        self
    }

    pub fn with_empty_selections(mut self, num_empty: u32) -> Self {
        self.num_empty_selections = num_empty;
        self
    }

    pub fn get_selected_image_l(&self) -> ImagePickerSelection {
        self.selected_image_l.get(&self.selection_set).copied().unwrap_or(ImagePickerSelection::None)
    }

    pub fn get_selected_image_r(&self) -> ImagePickerSelection {
        self.selected_image_r.get(&self.selection_set).copied().unwrap_or(ImagePickerSelection::None)
    }

    pub fn set_selected_image_l(&mut self, selected_image: ImagePickerSelection) {
        self.selected_image_l.insert(self.selection_set, selected_image);
        self.selected_image_l_changed = true;
    }

    pub fn set_selected_image_r(&mut self, selected_image: ImagePickerSelection) {
        self.selected_image_r.insert(self.selection_set, selected_image);
        self.selected_image_r_changed = true;
    }

    pub fn get_selected_image_l_for_set(&self, selection_set: usize) -> ImagePickerSelection {
        self.selected_image_l.get(&selection_set).copied().unwrap_or(ImagePickerSelection::None)
    }

    pub fn get_selected_image_r_for_set(&self, selection_set: usize) -> ImagePickerSelection {
        self.selected_image_r.get(&selection_set).copied().unwrap_or(ImagePickerSelection::None)
    }

    pub fn force_selection_into_visibility(&mut self) {
        self.selected_image_l_changed = true;
    }

    fn ui_pos_to_selection(&self, ui_pos: f32) -> ImagePickerSelection {
        if self.num_empty_selections != 0 {
            let num_empty = self.num_empty_selections as f32;
            let val = (ui_pos - num_empty).floor() as i32;
            if val == -1 {
                ImagePickerSelection::None
            } else if val < 0 {
                ImagePickerSelection::Empty(((-(val+2)) as u32).min(self.num_empty_selections.saturating_sub(2)))
            } else {
                ImagePickerSelection::Some(val as u32)
            }
        } else {
            ImagePickerSelection::Some(ui_pos.floor().max(0.0) as u32)
        }
    }

    fn selection_to_ui_pos(&self, selection: ImagePickerSelection) -> f32 {
        match selection {
            ImagePickerSelection::Some(index) => {
                index as f32 + self.num_empty_selections as f32
            }
            ImagePickerSelection::Empty(index) => {
                self.num_empty_selections as f32 - 2.0 - index as f32
            }
            ImagePickerSelection::None => {
                self.num_empty_selections as f32 - 1.0
            }
        }
    }

    fn draw_selection_rectangle(
        &self,
        painter: &egui::Painter,
        canvas_pos: Pos2,
        image_size: Vec2,
        selected_image: ImagePickerSelection,
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
        let bg_color = self.custom_bg_color.unwrap_or(
            if self.display.is_transparent() || self.num_empty_selections < 2 {
                wc.settings.image_bg_color
            } else {
                egui::Color32::from_rgb(0, 255, 0)
            }
        );
        let source = egui::scroll_area::ScrollSource {
            scroll_bar: true,
            drag: egui::scroll_area::DragScroll::Never,
            mouse_wheel: true,
        };
        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source);
        let zoom = self.zoom.unwrap_or(get_setting_zoom(wc.settings.tile_picker_zoom));
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
            let empty_item_size = zoom * Vec2::new(0.0, self.num_empty_selections as f32 * image.height() as f32);
            let image_picker_size = zoom * image.get_full_size() + empty_item_size + 2.0 * Vec2::splat(Self::BORDER);
            let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(16.0, 6.0)).min(Vec2::new(200.0, f32::INFINITY));
            let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
            let space = response.rect;
            let images_rect = Rect::from_min_size(
                space.min + empty_item_size + Vec2::splat(Self::BORDER),
                zoom * image.get_full_size()
            );
            let canvas_rect = Rect::from_min_size(space.min, image_picker_size);

            // draw background
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::BLACK);
            let image_item_size = image.get_item_size();
            if self.num_empty_selections != 0 {
                let empty_image_rect = Rect::from_min_size(
                    space.min + Vec2::splat(Self::BORDER),
                    zoom * Vec2::new(image_item_size.x, self.num_empty_selections as f32 * image_item_size.y)
                );
                painter.rect_filled(empty_image_rect, egui::CornerRadius::ZERO, bg_color);
            }
            painter.rect_filled(images_rect, egui::CornerRadius::ZERO, bg_color);

            // draw items
            let slot = image.texture_slot(self.display.is_transparent(), false);
            let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);
            egui::Image::from_texture((texture.id(), image_picker_size)).uv(super::FULL_UV).paint_at(ui, images_rect);
            if self.num_empty_selections > 1 {
                let image = STATIC_IMAGES.no_change_tile();
                let image_size = image.get_full_size();
                let image_draw_rect = Rect::from_min_size(
                    space.min + Vec2::splat(Self::BORDER),
                    zoom * image_size
                );
                let texture = image.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
                egui::Image::from_texture((texture.id(), image_size)).uv(super::FULL_UV).paint_at(ui, image_draw_rect);
            }

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
            let pos = pointer_pos - resp.inner_rect.min + resp.state.clamped_offset();
            if pos.x >= 0.0 && pos.x <= resp.inner_rect.width() {
                let frame_size = zoom * image.get_item_size();
                let num_items = image.num_items() as i32 + self.num_empty_selections as i32;
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
