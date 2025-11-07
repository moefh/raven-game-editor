use std::sync;
use crate::data_asset::DataAssetId;
use egui::{Vec2, Sense, Rect, Pos2};

const COLOR_PICKER_GRAD_COLORS: [u8; 6] = [
    0b110000,
    0b001100,
    0b000011,
    0b111100,
    0b001111,
    0b110011,
];
static COLOR_PICKER_GRAY_PAL: [u8; 4] = [ 0b000000, 0b010101, 0b101010, 0b111111 ];
static COLOR_PICKER_ALL_PAL: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(create_color_picker_all_pal);
static COLOR_PICKER_GRAD_PAL: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(create_color_picker_grad_pal);

pub struct ColorPickerState {
    pub left_color: u8,
    pub right_color: u8,
}

impl ColorPickerState {
    pub fn new(left_color: u8, right_color: u8) -> Self {
        ColorPickerState {
            left_color,
            right_color,
        }
    }
}

fn create_color_picker_all_pal() -> Vec<u8> {
    let mut colors = [0u8; 64];
    for r in 0..4 {
        for g in 0..4 {
            for b in 0..4 {
                let n = r * 16 + g * 4 + b;
                let x = n % 8;
                let y = (((n / 8) & 6) >> 1) | (((n / 8) & 1) << 2);
                let index = (y * 8 + x) as usize;
                colors[index] = ((r<<4) | (g<<2) | b) as u8
            }
        }
    }
    Vec::from(colors)
}

fn create_color_picker_grad_pal() -> Vec<u8> {
    let mut colors = [0u8; 6*5];

    for (c, &grad_color) in COLOR_PICKER_GRAD_COLORS.iter().enumerate() {
        let targets = [ grad_color, 0b111111 ];
        let mut x = 0;
        let mut color = 0b000000;
        for (i, &target) in targets.iter().enumerate() {
            let dcolor = if i == 0 { target & 0b010101 } else { (0b111111 & !color) & 0b010101 };
            for _ in 0..3 {
                if color != 0 && color != 0b111111 {  // don't show black
                    let r = (color & 0b110000) >> 4;
                    let g = (color & 0b001100) >> 2;
                    let b = color & 0b000011;
                    colors[c * 5 + x] = (r<<4) | (g<<2) | b;
                    x += 1;
                }
                color += dcolor;
            }
        }
    }
    Vec::from(colors)
}

pub fn draw_color_picker_palette(painter: &egui::Painter, rect: Rect, dims: (i32, i32), palette: &[u8]) {
    let item_w = rect.width() / (dims.0 as f32);
    let item_h = rect.height() / (dims.1 as f32);
    for y in 0..dims.1 {
        for x in 0..dims.0 {
            let item_rect = Rect {
                min: Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y + (y     as f32) * item_h),
                max: Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + ((y+1) as f32) * item_h),
            };
            let color_index = (y*dims.0+x) as usize;
            painter.rect_filled(item_rect, egui::CornerRadius::ZERO, crate::editors::raven_color_to_rgb(palette[color_index]));
        }
    }
}

fn check_color_picker_pick(pos: Pos2, rect: Rect, item: Vec2) -> Option<usize> {
    if ! rect.contains(pos) { return None; }
    let x = ((pos.x - rect.min.x) / item.x).floor() as i32;
    let y = ((pos.y - rect.min.y) / item.y).floor() as i32;
    let w = (rect.width() / item.x).round() as i32;
    Some((y * w + x) as usize)
}

fn set_color_picker_state_color(state: &mut ColorPickerState, color: u8, response: &egui::Response) {
    if response.dragged_by(egui::PointerButton::Primary) {
        state.left_color = color;
    } else if response.dragged_by(egui::PointerButton::Secondary) {
        state.right_color = color;
    }
}

pub fn color_picker(ui: &mut egui::Ui, asset_id: DataAssetId, state: &mut ColorPickerState) {
    ui.scope_builder(
        egui::UiBuilder::new().id_salt(format!("color_picker_{}", asset_id)),
        |ui| {
            let min_size = Vec2::splat(112.0).max(Vec2::new(112.0, ui.available_size().y));
            let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
            let full_rect = response.rect;
            let border = 8.0;
            let width = 8.0 * ((full_rect.width() - 2.0*border) / 8.0).floor();
            let left = response.rect.max.x - width - border;

            // selected colors
            //let sel_item_size = Vec2::new(width/2.0, width/4.0);
            let sel_start = Pos2::new(left, full_rect.min.y + border);
            let sel_rect = Rect::from_two_pos(sel_start, sel_start + Vec2::new(width, width/4.0));
            let sel_dims = (2, 1);

            // all colors palette
            let all_item_size = Vec2::splat(width / 8.0);
            let all_start = Pos2::new(left, sel_rect.max.y + border);
            let all_rect = Rect {
                min: all_start,
                max: all_start + 8.0 * all_item_size,
            };
            let all_dims = (8, 8);

            // grays palette
            let grays_item_size = Vec2::new(width / 4.0, all_item_size.y);
            let grays_start = Pos2::new(left, all_rect.max.y + border);
            let grays_rect = Rect {
                min: grays_start,
                max: grays_start + Vec2::new(4.0 * grays_item_size.x, grays_item_size.y),
            };
            let grays_dims = (4, 1);

            // gradients palette
            let grads_item_size = all_item_size;
            let grads_start = Pos2::new(left + (width - 5.0*grads_item_size.x) / 2.0, grays_rect.max.y + border);
            let grads_rect = Rect {
                min: grads_start,
                max: grads_start + Vec2::new(5.0 * grads_item_size.x, 6.0 * grads_item_size.y),
            };
            let grads_dims = (5, 6);

            let bg_rect = full_rect.with_max_y(grads_rect.max.y + border);

            painter.rect_filled(bg_rect, egui::CornerRadius::same(8), egui::Color32::from_rgb(224, 224, 224));
            draw_color_picker_palette(&painter, sel_rect, sel_dims, &[state.left_color, state.right_color]);
            draw_color_picker_palette(&painter, all_rect, all_dims, &COLOR_PICKER_ALL_PAL);
            draw_color_picker_palette(&painter, grays_rect, grays_dims, &COLOR_PICKER_GRAY_PAL);
            draw_color_picker_palette(&painter, grads_rect, grads_dims, &COLOR_PICKER_GRAD_PAL);

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if let Some(index) = check_color_picker_pick(pointer_pos, all_rect, all_item_size) &&
                    let Some(&color) = COLOR_PICKER_ALL_PAL.get(index) {
                        set_color_picker_state_color(state, color, &response);
                    }
                if let Some(index) = check_color_picker_pick(pointer_pos, grays_rect, grays_item_size) &&
                    let Some(&color) = COLOR_PICKER_GRAY_PAL.get(index) {
                        set_color_picker_state_color(state, color, &response);
                    }
                if let Some(index) = check_color_picker_pick(pointer_pos, grads_rect, grads_item_size) &&
                    let Some(&color) = COLOR_PICKER_GRAD_PAL.get(index) {
                        set_color_picker_state_color(state, color, &response);
                    }
            }
        });
}
