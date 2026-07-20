use std::sync;
use egui::{Vec2, Sense, Rect, Pos2};

use super::ColorPickerPopupWidget;

use crate::app::WindowContext;
use crate::image::colors::{
    color_to_rgb,
    color_to_rgb_contrast,
};

pub enum ColorPickerResponse {
    None,
    CreateColorset,
}

const CLOSE_PICKER_ON_CLICK: bool = true;
const MIN_PICKER_WIDTH: f32 = 96.0;

const GRAY_PAL6: [u8; 4] = [
    0b00_000_000,
    0b01_010_010,
    0b10_101_101,
    0b11_111_111,
];
const GRAD_PAL6: [u8; 30] = [
    0b01_000_000,0b10_000_000,0b11_000_000,0b11_010_010,0b11_101_101,
    0b00_010_000,0b00_101_000,0b00_111_000,0b01_111_010,0b10_111_101,
    0b00_000_010,0b00_000_101,0b00_000_111,0b01_010_111,0b10_101_111,
    0b01_010_000,0b10_101_000,0b11_111_000,0b11_111_010,0b11_111_101,
    0b00_010_010,0b00_101_101,0b00_111_111,0b01_111_111,0b10_111_111,
    0b01_000_010,0b10_000_101,0b11_000_111,0b11_010_111,0b11_101_111,
];

static ALL_PAL6: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(|| {
    let mut colors = [0u8; 64];
    for r in 0..4 {
        for g in 0..4 {
            for b in 0..4 {
                let n = r * 16 + g * 4 + b;
                let x = n % 8;
                let y = (((n / 8) & 6) >> 1) | (((n / 8) & 1) << 2);
                let index = (y * 8 + x) as usize;
                colors[index] = ((b<<6) | (g<<4) | (g >> 1) | (r << 1) | (r >> 1)) as u8
            }
        }
    }
    Vec::from(colors)
});

struct Color6PickerWidget<'a> {
    state: &'a mut ColorPickerState,
}

impl<'a> Color6PickerWidget<'a> {
    fn new(state: &'a mut ColorPickerState) -> Self {
        Color6PickerWidget {
            state,
        }
    }

    fn draw_palette(painter: &egui::Painter, rect: Rect, dims: (i32, i32), palette: &[u8]) {
        let item_w = rect.width() / (dims.0 as f32);
        let item_h = rect.height() / (dims.1 as f32);
        for y in 0..dims.1 {
            for x in 0..dims.0 {
                let item_rect = Rect {
                    min: Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y + (y     as f32) * item_h),
                    max: Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + ((y+1) as f32) * item_h),
                };
                let color_index = (y*dims.0+x) as usize;
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(palette[color_index]));
            }
        }
    }

    fn check_pick(pos: Pos2, rect: Rect, item: Vec2) -> Option<usize> {
        if ! rect.contains(pos) { return None; }
        let x = ((pos.x - rect.min.x) / item.x).floor() as i32;
        let y = ((pos.y - rect.min.y) / item.y).floor() as i32;
        let w = (rect.width() / item.x).round() as i32;
        Some((y * w + x) as usize)
    }

    fn set_color(&mut self, color: u8, response: &egui::Response) {
        if response.dragged_by(egui::PointerButton::Primary) {
            self.state.left_color = color;
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            self.state.right_color = color;
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext) {
        let min_size = Vec2::splat(MIN_PICKER_WIDTH).max(Vec2::new(MIN_PICKER_WIDTH, ui.available_size().y));
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

        painter.rect_filled(bg_rect, egui::CornerRadius::same(8), wc.settings.color_picker_bg_color);
        Self::draw_palette(&painter, sel_rect, sel_dims, &[self.state.left_color, self.state.right_color]);
        Self::draw_palette(&painter, all_rect, all_dims, &ALL_PAL6);
        Self::draw_palette(&painter, grays_rect, grays_dims, &GRAY_PAL6);
        Self::draw_palette(&painter, grads_rect, grads_dims, &GRAD_PAL6);

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if let Some(index) = Self::check_pick(pointer_pos, all_rect, all_item_size) &&
                let Some(&color) = ALL_PAL6.get(index) {
                    self.set_color(color, &response);
                }
            if let Some(index) = Self::check_pick(pointer_pos, grays_rect, grays_item_size) &&
                let Some(&color) = GRAY_PAL6.get(index) {
                    self.set_color(color, &response);
                }
            if let Some(index) = Self::check_pick(pointer_pos, grads_rect, grads_item_size) &&
                let Some(&color) = GRAD_PAL6.get(index) {
                    self.set_color(color, &response);
                }
        }
    }
}

struct Color8PickerWidget<'a> {
    state: &'a mut ColorPickerState,
    popup: ColorPickerPopupWidget,
    allow_new_colorset: bool,
    response: ColorPickerResponse,
}

impl<'a> Color8PickerWidget<'a> {
    fn new(state: &'a mut ColorPickerState, popup_id: egui::Id, allow_new_colorset: bool) -> Self {
        Color8PickerWidget {
            state,
            allow_new_colorset,
            response: ColorPickerResponse::None,
            popup: ColorPickerPopupWidget::new(popup_id, CLOSE_PICKER_ON_CLICK),
        }
    }

    fn get_open_color(&mut self) -> Option<u8> {
        match self.state.open_color {
            OpenColor::Left => Some(self.state.left_color),
            OpenColor::Right => Some(self.state.right_color),
            OpenColor::None => None,
        }
    }

    fn set_open_color(&mut self, color: u8) {
        match self.state.open_color {
            OpenColor::Left => { self.state.left_color = color; }
            OpenColor::Right => { self.state.right_color = color; }
            OpenColor::None => {}
        }
    }

    fn draw_selected_colors(&self, painter: &egui::Painter, rect: Rect) {
        let item_w = rect.width() / 2.0;
        let item_h = rect.height();
        let colors = [ self.state.left_color, self.state.right_color ];
        for (x, &color) in colors.iter().enumerate() {
            let item_rect = Rect {
                min: Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y         ),
                max: Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + item_h),
            };
            painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
        }
    }

    fn draw_colorset(&self, painter: &egui::Painter, rect: Rect, colors: &[u8]) {
        let item_size = rect.width() / 4.0;
        for (i, &color) in colors.iter().enumerate() {
            let x = i % 4;
            let y = i / 4;
            let item_rect = Rect {
                min: Pos2::new(rect.min.x + (x     as f32) * item_size, rect.min.y + (y       as f32) * item_size),
                max: Pos2::new(rect.min.x + ((x+1) as f32) * item_size, rect.min.y + ((y + 1) as f32) * item_size),
            };
            if item_rect.min.y > rect.max.y {
                break;
            }
            painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
            if self.state.left_color == color || self.state.right_color == color {
                painter.rect_stroke(item_rect, 0.0, egui::Stroke::new(2.0, color_to_rgb_contrast(color)), egui::StrokeKind::Inside);
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, colorset_combo_id: egui::Id) {
        self.response = ColorPickerResponse::None;
        egui::Frame::NONE
            .inner_margin(8.0)
            .corner_radius(egui::CornerRadius::same(8))
            .fill(wc.settings.color_picker_bg_color)
            .show(ui, |ui| {
                let min_size = Vec2::new(MIN_PICKER_WIDTH - 16.0, 24.0);

                // selected colors
                let (response, painter) = ui.allocate_painter(min_size, Sense::click());
                self.draw_selected_colors(&painter, response.rect);
                if response.clicked() && let Some(pos) = response.interact_pointer_pos() {
                    self.state.open_color = if pos.x <= response.rect.center().x { OpenColor::Left } else { OpenColor::Right };
                };
                if let Some(edit_color) = self.get_open_color() {
                    let mut edit_color = edit_color;
                    self.popup.show(&response, wc.settings, &mut edit_color);
                    self.set_open_color(edit_color);
                    if self.popup.close {
                        self.state.open_color = OpenColor::None;
                    }
                }

                // color set
                if let Some(colorset_colors) = wc.settings.colorsets.get_colorset_colors(self.state.colorset) {
                    let color_block_size = (response.rect.width() / 4.0).floor();
                    let min_size = Vec2::new(response.rect.width(),
                                             color_block_size * colorset_colors.len().div_ceil(4).max(7) as f32);

                    ui.add_space(8.0);
                    let (response, painter) = ui.allocate_painter(min_size, Sense::drag());

                    self.draw_colorset(&painter, response.rect, colorset_colors);
                    if response.dragged() && let Some(pos) = response.interact_pointer_pos() {
                        let x = ((pos.x - response.rect.min.x) / color_block_size).floor().clamp(0.0, 3.0) as usize;
                        let y = ((pos.y - response.rect.min.y) / color_block_size).floor().max(0.0) as usize;
                        if let Some(&color) = colorset_colors.get(y*4 + x) {
                            if response.dragged_by(egui::PointerButton::Primary) {
                                self.state.left_color = color;
                            } else if response.dragged_by(egui::PointerButton::Secondary) {
                                self.state.right_color = color;
                            }
                        }
                    }
                }

                ui.add_space(8.0);
                egui::ComboBox::from_id_salt(colorset_combo_id)
                    .selected_text(wc.settings.colorsets.get_colorset_name(self.state.colorset).unwrap_or("--"))
                    .width(response.rect.width())
                    .truncate()
                    .show_ui(ui, |ui| {
                        for (index, name) in wc.settings.colorsets.get_colorset_names().enumerate() {
                            ui.selectable_value(&mut self.state.colorset, index, name);
                        }
                    });
                ui.horizontal(|ui| {
                    if ui.add_enabled(wc.settings.colorsets.is_colorset_custom(self.state.colorset),
                                      egui::Button::new("Edit")
                                      .min_size(Vec2::new(response.rect.width(), 10.0))
                                      .truncate()).clicked() {
                        wc.open_colorset_dialog(self.state.colorset);
                    }
                });
                if self.allow_new_colorset {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("New").min_size(Vec2::new(response.rect.width(), 10.0)).truncate()).clicked() {
                            self.response = ColorPickerResponse::CreateColorset;
                        }
                    });
                }
                ui.add_space(0.0);
            });
    }
}

pub enum OpenColor {
    None,
    Left,
    Right,
}

pub struct ColorPickerState {
    pub left_color: u8,
    pub right_color: u8,
    pub colorset: usize,
    open_color: OpenColor,
}

pub struct ColorPickerWidget {
    pub state: ColorPickerState,
    colorset_combo_id: egui::Id,
    popup_id: egui::Id,
    allow_new_colorset: bool,
}

impl ColorPickerWidget {
    pub fn new(id_prefix: impl AsRef<str>, left_color: u8, right_color: u8, allow_new_colorset: bool) -> Self {
        ColorPickerWidget {
            colorset_combo_id: egui::Id::new(format!("{}_combo", id_prefix.as_ref())),
            popup_id: egui::Id::new(format!("{}_popup", id_prefix.as_ref())),
            allow_new_colorset,
            state: ColorPickerState {
                left_color,
                right_color,
                open_color: OpenColor::None,
                colorset: 0,
            },
        }
    }

    pub fn set_colorset(&mut self, colorset: usize) {
        self.state.colorset = colorset;
    }

    pub fn maybe_set_colors(&mut self, left_color: Option<u8>, right_color: Option<u8>) {
        if let Some(color) = left_color {
            self.state.left_color = color;
        }
        if let Some(color) = right_color {
            self.state.right_color = color;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext) -> ColorPickerResponse {
        match wc.vga_bits_per_pixel {
            8 => {
                let mut picker = Color8PickerWidget::new(&mut self.state, self.popup_id, self.allow_new_colorset);
                picker.show(ui, wc, self.colorset_combo_id);
                picker.response
            }
            _ => {
                let mut picker = Color6PickerWidget::new(&mut self.state);
                picker.show(ui, wc);
                ColorPickerResponse::None
            }
        }
    }
}
