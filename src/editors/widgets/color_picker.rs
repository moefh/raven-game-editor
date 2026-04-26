use std::sync;
use egui::{Vec2, Sense, Rect, Pos2};

use crate::app::WindowContext;

const CLOSE_PICKER_ON_CLICK: bool = false;
const MIN_PICKER_WIDTH: f32 = 112.0;
const COLOR_BLOCK_SIZE: f32 = 20.0;

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

static ALL_PAL6: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(create_all_pal6);
fn create_all_pal6() -> Vec<u8> {
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
}

const GRAY_PAL8: [u8; 8] = [
    0b00_000_000,
    0b00_001_001,
    0b01_010_010,
    0b01_011_011,
    0b10_100_100,
    0b10_101_101,
    0b11_110_110,
    0b11_111_111
];
const GRAD_PAL8: [u8; 78] = [
    // blue
    0b01_000_000, 0b01_000_000, 0b01_000_000, 0b10_000_000, 0b10_000_000, 0b10_000_000,
    0b11_000_000,
    0b11_001_001, 0b11_010_010, 0b11_011_011, 0b11_100_100, 0b11_101_101, 0b11_110_110,

    // green
    0b00_001_000, 0b00_010_000, 0b00_011_000, 0b00_100_000, 0b00_101_000, 0b00_110_000,
    0b00_111_000,
    0b00_111_001, 0b00_111_010, 0b01_111_011, 0b01_111_100, 0b10_111_101, 0b10_111_110,

    // red
    0b00_000_001, 0b00_000_010, 0b00_000_011, 0b00_000_100, 0b00_000_101, 0b00_000_110,
    0b00_000_111,
    0b00_001_111, 0b00_010_111, 0b01_011_111, 0b01_100_111, 0b10_101_111, 0b10_110_111,

    // cyan
    0b01_001_000, 0b01_010_000, 0b01_011_000, 0b10_100_000, 0b10_101_000, 0b10_110_000,
    0b11_111_000,
    0b11_111_001, 0b11_111_010, 0b11_111_011, 0b11_111_100, 0b11_111_101, 0b11_111_110,

    // yellow
    0b00_001_001, 0b00_010_010, 0b00_011_011, 0b00_100_100, 0b00_101_101, 0b00_110_110,
    0b00_111_111,
    0b00_111_111, 0b00_111_111, 0b01_111_111, 0b01_111_111, 0b10_111_111, 0b10_111_111,

    // magenta
    0b01_000_001, 0b01_000_010, 0b01_000_011, 0b01_000_100, 0b10_000_101, 0b10_000_110,
    0b11_000_111,
    0b11_001_111, 0b11_010_111, 0b11_011_111, 0b11_100_111, 0b11_101_111, 0b11_110_111,
];

static ALL_PAL8: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(create_all_pal8);
fn create_all_pal8() -> Vec<u8> {
    let mut colors = [0u8; 256];
    let mut index = 0usize;
    for r in 0..8 {
        for b in 0..4 {
            for g in 0..8 {
                colors[index] = r | (g << 3) | (b << 6);
                index += 1;
            }
        }
    }
    Vec::from(colors)
}

struct ColorPickerWidget6<'a> {
    state: &'a mut ColorPickerState,
}

impl<'a> ColorPickerWidget6<'a> {
    fn new(state: &'a mut ColorPickerState) -> Self {
        ColorPickerWidget6 {
            state,
        }
    }

    pub fn color_to_rgb(color: u8) -> egui::Color32 {
        let r = (color >> 1) & 0x3;
        let g = (color >> 4) & 0x3;
        let b = (color >> 6) & 0x3;
        let cr = (r << 6) | (r << 4) | (r << 2) | r;
        let cg = (g << 6) | (g << 4) | (g << 2) | g;
        let cb = (b << 6) | (b << 4) | (b << 2) | b;
        egui::Color32::from_rgb(cr, cg, cb)
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
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, Self::color_to_rgb(palette[color_index]));
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

struct ColorPickerWidget8<'a> {
    state: &'a mut ColorPickerState,
}

impl<'a> ColorPickerWidget8<'a> {
    fn new(state: &'a mut ColorPickerState) -> Self {
        ColorPickerWidget8 {
            state,
        }
    }

    fn get_open_color(&mut self) -> u8 {
        match self.state.open_color {
            Some(OpenColor::Left) => self.state.left_color ,
            Some(OpenColor::Right) => self.state.right_color,
            _ => 0,
        }
    }

    fn set_open_color(&mut self, color: u8) {
        match self.state.open_color {
            Some(OpenColor::Left) => { self.state.left_color = color; }
            Some(OpenColor::Right) => { self.state.right_color = color; }
            _ => {}
        }
    }

    fn get_contrast_color(color: u8) -> egui::Color32 {
        let r = (color & 0x7) as f32;
        let g = ((color >> 3) & 0x7) as f32;
        let b = (((color >> 5) & 0x6) | (color >> 7)) as f32;
        let brightness = r*0.3 + g*0.8 + b*0.1;
        if brightness < 4.5 {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        }
    }

    pub fn color_to_rgb(color: u8) -> egui::Color32 {
        let r = color & 0x7;
        let g = (color >> 3) & 0x7;
        let b = (color >> 6) & 0x3;
        let cr = (r << 5) | (r << 2) | (r >> 2);
        let cg = (g << 5) | (g << 2) | (g >> 2);
        let cb = (b << 6) | (b << 4) | (b << 2) | b;
        egui::Color32::from_rgb(cr, cg, cb)
    }

    fn draw_palette(painter: &egui::Painter, rect: Rect, dims: (i32, i32), palette: &[u8], sel_color: Option<u8>) {
        let item_w = rect.width() / (dims.0 as f32);
        let item_h = rect.height() / (dims.1 as f32);
        for y in 0..dims.1 {
            for x in 0..dims.0 {
                let item_rect = Rect {
                    min: Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y + (y     as f32) * item_h),
                    max: Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + ((y+1) as f32) * item_h),
                };
                let color = palette[(y*dims.0+x) as usize];
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, Self::color_to_rgb(color));
                if let Some(sel_color) = sel_color && sel_color == color {
                    let stroke = egui::Stroke::new(1.0, Self::get_contrast_color(color));
                    painter.rect_stroke(item_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
                }
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext, id: egui::Id) {
        egui::Frame::NONE
            .inner_margin(8.0)
            .corner_radius(egui::CornerRadius::same(8))
            .fill(wc.settings.color_picker_bg_color)
            .show(ui, |ui| {
                let min_size = Vec2::new(MIN_PICKER_WIDTH - 16.0, 24.0);
                let (response, painter) = ui.allocate_painter(min_size, Sense::click());
                Self::draw_palette(&painter, response.rect, (2, 1), &[self.state.left_color, self.state.right_color], None);

                if response.clicked() && let Some(pos) = response.interact_pointer_pos() {
                    self.state.open_color = Some(if pos.x <= response.rect.center().x { OpenColor::Left } else { OpenColor::Right });
                };

                let maybe_inner = egui::containers::Popup::menu(&response)
                    .id(id)
                    .open(self.state.open_color.is_some())
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        // RGB values
                        ui.horizontal(|ui| {
                            let open_color = self.get_open_color();
                            let mut r = open_color & 7;
                            let mut g = (open_color >> 3) & 7;
                            let mut b = (open_color >> 6) & 3;
                            ui.add(egui::DragValue::new(&mut r).prefix("R ").speed(0.07).range(0..=7));
                            ui.add(egui::DragValue::new(&mut g).prefix("G ").speed(0.07).range(0..=7));
                            ui.add(egui::DragValue::new(&mut b).prefix("B ").speed(0.07).range(0..=3));
                            ui.label(format!("binary: {:08b}  hex: {:02x}", open_color, open_color));
                            self.set_open_color(r | (g << 3) | (b << 6));
                        });

                        // full palette
                        ui.horizontal(|ui| {
                            let size = Vec2::new(32.0 * COLOR_BLOCK_SIZE, 8.0 * COLOR_BLOCK_SIZE);
                            let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
                            Self::draw_palette(&painter, response.rect, (32, 8), &ALL_PAL8, Some(self.get_open_color()));
                            if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                                let x = ((pos.x - response.rect.min.x) / COLOR_BLOCK_SIZE).floor() as i32;
                                let y = ((pos.y - response.rect.min.y) / COLOR_BLOCK_SIZE).floor() as i32;
                                let index = y * 32 + x;
                                if index >= 0 && index <= ALL_PAL8.len() as i32 {
                                    self.set_open_color(ALL_PAL8[index as usize]);
                                }
                            }
                            if (CLOSE_PICKER_ON_CLICK && response.clicked()) || response.double_clicked() {
                                self.state.open_color = None;
                            }
                        });

                        ui.add_space(8.0);

                        egui::Frame::NONE
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(4.0)
                            .fill(wc.settings.color_picker_bg_color)
                            .show(ui, |ui| {
                                // grays palette
                                ui.horizontal(|ui| {
                                    let num_colors = GRAY_PAL8.len() as i32;
                                    let h_color_block_size = COLOR_BLOCK_SIZE * 13.0 / (num_colors as f32);
                                    let alloc_size = Vec2::new(32.0 * COLOR_BLOCK_SIZE - 6.0, COLOR_BLOCK_SIZE);
                                    let (response, painter) = ui.allocate_painter(alloc_size, Sense::click_and_drag());
                                    let pal_width = h_color_block_size * num_colors as f32;
                                    let draw_rect = egui::Rect {
                                        min: egui::Pos2::new(response.rect.center().x - pal_width/2.0, response.rect.min.y),
                                        max: egui::Pos2::new(response.rect.center().x + pal_width/2.0, response.rect.max.y),
                                    };
                                    Self::draw_palette(&painter, draw_rect, (num_colors, 1), &GRAY_PAL8, Some(self.get_open_color()));
                                    if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                                        let index = ((pos.x - draw_rect.min.x) / h_color_block_size).floor() as i32;
                                        if index >= 0 && index < num_colors {
                                            self.set_open_color(GRAY_PAL8[index as usize]);
                                        }
                                    }
                                    if (CLOSE_PICKER_ON_CLICK && response.clicked()) || response.double_clicked() {
                                        self.state.open_color = None;
                                    }
                                });

                                ui.add_space(2.0);

                                // gradients palette
                                ui.horizontal(|ui| {
                                    let alloc_size = Vec2::new(32.0 * COLOR_BLOCK_SIZE - 6.0, 6.0 * COLOR_BLOCK_SIZE);
                                    let (response, painter) = ui.allocate_painter(alloc_size, Sense::click_and_drag());
                                    let pal_width = COLOR_BLOCK_SIZE * 13.0;
                                    let draw_rect = egui::Rect {
                                        min: egui::Pos2::new(response.rect.center().x - pal_width/2.0, response.rect.min.y),
                                        max: egui::Pos2::new(response.rect.center().x + pal_width/2.0, response.rect.max.y),
                                    };
                                    Self::draw_palette(&painter, draw_rect, (13, 6), &GRAD_PAL8, Some(self.get_open_color()));
                                    if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                                        let x = ((pos.x - draw_rect.min.x) / COLOR_BLOCK_SIZE).floor() as i32;
                                        let y = ((pos.y - draw_rect.min.y) / COLOR_BLOCK_SIZE).floor() as i32;
                                        let index = y * 13 + x;
                                        if (0..13).contains(&x) && y >= 0 && index >= 0 && index < GRAD_PAL8.len() as i32 {
                                            self.set_open_color(GRAD_PAL8[index as usize]);
                                        }
                                    }
                                    if (CLOSE_PICKER_ON_CLICK && response.clicked()) || response.double_clicked() {
                                        self.state.open_color = None;
                                    }
                                });
                            });
                    });
                if let Some(inner) = maybe_inner && inner.response.should_close() {
                    self.state.open_color = None;
                }
            });
    }
}

enum OpenColor {
    Left,
    Right,
}

pub struct ColorPickerState {
    pub left_color: u8,
    pub right_color: u8,
    open_color: Option<OpenColor>,
}

pub struct ColorPickerWidget {
    pub state: ColorPickerState,
}

impl ColorPickerWidget {
    pub fn new(left_color: u8, right_color: u8) -> Self {
        ColorPickerWidget {
            state: ColorPickerState {
                left_color,
                right_color,
                open_color: None,
            },
        }
    }

    pub fn maybe_set_colors(&mut self, left_color: Option<u8>, right_color: Option<u8>) {
        if let Some(color) = left_color {
            self.state.left_color = color;
        }
        if let Some(color) = right_color {
            self.state.right_color = color;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &WindowContext, id: impl Into<egui::Id>) {
        match wc.vga_bits_per_pixel {
            8 => {
                let mut picker = ColorPickerWidget8::new(&mut self.state);
                picker.show(ui, wc, id.into());
            }
            _ => {
                let mut picker = ColorPickerWidget6::new(&mut self.state);
                picker.show(ui, wc);
            }
        }
    }
}
