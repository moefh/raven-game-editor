use egui::{Vec2, Sense, Rect, Pos2};

use crate::app::WindowContext;
use crate::image::colors::{
    color_to_rgb,
    color_to_rgb_contrast,
};

const MIN_PICKER_WIDTH: f32 = 112.0;

pub enum PalColorPickerAction {
    None,
    EditPalette,
    ChangeSelectedLeftColor,
    ChangeSelectedRightColor,
}

pub struct PalColorPickerState {
    pub left_index: u8,
    pub right_index: u8,
}

pub struct PalColorPickerWidget {
    pub state: PalColorPickerState,
}

impl PalColorPickerWidget {
    pub fn new(left_index: u8, right_index: u8) -> Self {
        PalColorPickerWidget {
            state: PalColorPickerState {
                left_index,
                right_index,
            },
        }
    }

    pub fn maybe_set_colors(&mut self, left_color: Option<u8>, right_color: Option<u8>, color_to_palette_index_map: &[u8]) {
        if let Some(color) = left_color {
            self.state.left_index = color_to_palette_index_map[color as usize];
        }
        if let Some(color) = right_color {
            self.state.right_index = color_to_palette_index_map[color as usize];
        }
    }

    fn set_color_index(&mut self, index: u8, response: &egui::Response) -> PalColorPickerAction {
        if response.dragged_by(egui::PointerButton::Primary) {
            self.state.left_index = index;
            PalColorPickerAction::ChangeSelectedLeftColor
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            self.state.right_index = index;
            PalColorPickerAction::ChangeSelectedRightColor
        } else {
            PalColorPickerAction::None
        }
    }

    fn check_pick(pos: Pos2, rect: Rect, item: Vec2) -> Option<usize> {
        if ! rect.contains(pos) { return None; }
        let x = ((pos.x - rect.min.x) / item.x).floor() as i32;
        let y = ((pos.y - rect.min.y) / item.y).floor() as i32;
        let w = (rect.width() / item.x).round() as i32;
        Some((y * w + x) as usize)
    }

    fn draw_palette(&self, painter: &egui::Painter, rect: Rect, dims: (i32, i32), palette: &[u8], draw_selection: bool) {
        let item_w = rect.width() / (dims.0 as f32);
        let item_h = rect.height() / (dims.1 as f32);
        for y in 0..dims.1 {
            for x in 0..dims.0 {
                let item_rect = Rect {
                    min: Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y + (y     as f32) * item_h),
                    max: Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + ((y+1) as f32) * item_h),
                };
                let color_index = (y*dims.0+x) as usize;
                let color = if color_index < palette.len() { palette[color_index] } else { 0 };
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
                if draw_selection && (self.state.left_index as usize == color_index || self.state.right_index as usize == color_index) {
                    painter.rect_stroke(item_rect, 0.0, egui::Stroke::new(1.0, color_to_rgb_contrast(color)), egui::StrokeKind::Inside);
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext, palette: &[u8]) -> PalColorPickerAction {
        let min_size = Vec2::splat(MIN_PICKER_WIDTH).max(Vec2::new(MIN_PICKER_WIDTH, ui.available_size().y));
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let full_rect = response.rect;
        let border = 8.0;
        let width = 8.0 * ((full_rect.width() - 2.0*border) / 8.0).floor();
        let left = response.rect.max.x - width - border;

        // selected colors
        let sel_start = Pos2::new(left, full_rect.min.y + border);
        let sel_rect = Rect::from_two_pos(sel_start, sel_start + Vec2::new(width, width/4.0));
        let sel_dims = (2, 1);
        let sel_colors = &[
            palette[self.state.left_index as usize % palette.len()],
            palette[self.state.right_index as usize % palette.len()],
        ];

        // edit palette button
        let edit_pal_rect = Rect::from_min_size(Pos2::new(left, sel_rect.max.y + 2.0 * border), Vec2::new(width, 24.0));

        // palette
        let (pal_colors_x, pal_colors_y) = if palette.len() <= 8 {
            (palette.len() as i32, 1)
        } else {
            let h = palette.len() / 8;
            ((palette.len() / h) as i32, h as i32)
        };
        let pal_item_size = Vec2::splat(width / pal_colors_x as f32);
        let pal_start = Pos2::new(left, edit_pal_rect.max.y + border);
        let pal_rect = Rect {
            min: pal_start,
            max: pal_start + Vec2::new(pal_colors_x as f32 * pal_item_size.x, pal_colors_y as f32 * pal_item_size.y),
        };
        let pal_dims = (pal_colors_x, pal_colors_y);

        let bg_rect = full_rect.with_max_y(pal_rect.max.y + border);

        // show stuff:
        let mut action = PalColorPickerAction::None;

        painter.rect_filled(bg_rect, egui::CornerRadius::same(8), wc.settings.color_picker_bg_color);
        self.draw_palette(&painter, sel_rect, sel_dims, sel_colors, false);
        if ui.place(edit_pal_rect, egui::Button::new("Edit Palette")).clicked() {
            action = PalColorPickerAction::EditPalette;
        }
        self.draw_palette(&painter, pal_rect, pal_dims, palette, true);

        if let Some(pointer_pos) = response.interact_pointer_pos() &&
            let Some(index) = Self::check_pick(pointer_pos, pal_rect, pal_item_size) {
                action = self.set_color_index((index & 0xff) as u8, &response);
            }

        action
    }
}
