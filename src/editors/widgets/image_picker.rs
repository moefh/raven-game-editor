use egui::{Vec2, Pos2, Color32, Rect};

use crate::app::WindowContext;
use crate::misc::ImageCollection;
use super::image_editor::ImageDisplay;
use crate::data_asset::ImageCollectionAsset;

pub struct ImagePickerState {
    pub allow_empty_selection: bool,
    pub allow_second_selection: bool,
    pub selected_image: u32,
    pub selected_image_right: u32,
    pub zoom: f32,
    pub background_color: Option<Color32>,
    pub display: ImageDisplay,
}

impl ImagePickerState {
    pub fn new() -> Self {
        ImagePickerState {
            allow_empty_selection: false,
            allow_second_selection: false,
            zoom: 1.0,
            selected_image: 0,
            selected_image_right: 0xff,
            background_color: None,
            display: ImageDisplay::new(0),
        }
    }

    pub fn use_as_palette(mut self, use_as_palette: bool) -> Self {
        self.allow_second_selection = use_as_palette;
        self.allow_empty_selection = use_as_palette;
        self.background_color = Some(Color32::from_rgb(0, 0xff, 0));
        self
    }

    fn sel_index_to_image(&self, sel_index: u32) -> u32 {
        if self.allow_empty_selection {
            if sel_index == 0 { 0xff} else { sel_index - 1 }
        } else {
            sel_index
        }
    }

    fn image_to_sel_index(&self, image_index: u32) -> u32 {
        if self.allow_empty_selection {
            if image_index == 0xff { 0 } else { image_index + 1 }
        } else {
            image_index
        }
    }

    fn draw_selection_rectangle(&self, painter: &egui::Painter, canvas_pos: Pos2, image_size: Vec2,
                                selected_image: u32, color1: Color32, color2: Color32) {
        let pos = canvas_pos + Vec2::new(0.0, (self.image_to_sel_index(selected_image) as f32) * image_size.y);
        let sel_rect = Rect {
            min: pos,
            max: pos + image_size,
        };
        let stroke = egui::Stroke::new(3.0, color1);
        painter.rect_stroke(sel_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);

        let in_stroke = egui::Stroke::new(1.0, color2);
        let sel_in_rect = sel_rect.expand2(Vec2::splat(-2.0));
        painter.rect_stroke(sel_in_rect, egui::CornerRadius::ZERO, in_stroke, egui::StrokeKind::Inside);
    }
}

pub fn image_picker(ui: &mut egui::Ui, wc: &mut WindowContext, asset: &impl ImageCollectionAsset, state: &mut ImagePickerState) {
    let (image, texture) = ImageCollection::load_asset_texture(asset, wc.tex_man, wc.egui.ctx, state.display.texture_slot(), false);
    let source = egui::scroll_area::ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true };
    let scroll = egui::ScrollArea::vertical().auto_shrink([true, true]).scroll_source(source).show(ui, |ui| {
        let image_size = state.zoom * image.get_item_size();
        let empty_item_size = state.zoom * if state.allow_empty_selection { Vec2::new(0.0, image.height as f32) } else { Vec2::ZERO };
        let image_picker_size = state.zoom * image.get_full_size() + empty_item_size;
        let min_size = Vec2::splat(50.0).max(image_picker_size + Vec2::new(16.0, 6.0)).min(Vec2::new(200.0, f32::INFINITY));
        let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
        let space = response.rect;
        let images_rect = Rect {
            min: space.min + empty_item_size,
            max: space.min + empty_item_size + state.zoom * image.get_full_size(),
        };
        let canvas_rect = Rect {
            min: space.min,
            max: space.min + image_picker_size,
        };

        // draw background
        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, state.background_color.unwrap_or(wc.settings.image_bg_color));

        // draw items
        egui::Image::from_texture((texture.id(), image_picker_size)).uv(super::FULL_UV).paint_at(ui, images_rect);

        // draw selection rectangles
        state.draw_selection_rectangle(&painter, canvas_rect.min, image_size, state.selected_image,
                                       egui::Color32::BLUE, egui::Color32::WHITE);
        if state.allow_second_selection {
            state.draw_selection_rectangle(&painter, canvas_rect.min, image_size, state.selected_image_right,
                                           egui::Color32::RED, egui::Color32::WHITE);
        }

        response
    });
    if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
        let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
        if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
            let frame_size = state.zoom * image.get_item_size();
            let num_items = image.num_items + if state.allow_empty_selection { 1 } else { 0 };
            let image_index = state.sel_index_to_image(u32::min((pos.y / frame_size.y).floor() as u32, num_items - 1));
            if scroll.inner.dragged_by(egui::PointerButton::Primary) {
                state.selected_image = image_index;
            } else if scroll.inner.dragged_by(egui::PointerButton::Secondary) {
                state.selected_image_right = image_index;
            }
        }
    };
}
