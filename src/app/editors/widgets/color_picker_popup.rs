use std::sync;
use egui::{Vec2, Sense, Rect};

use crate::app::AppSettings;
use crate::image::colors::{color_to_rgb, color_to_rgb_contrast};

const COLOR_BLOCK_SIZE: f32 = 20.0;

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
    0b01_111_111, 0b01_111_111, 0b01_111_111, 0b10_111_111, 0b10_111_111, 0b10_111_111,

    // magenta
    0b01_000_001, 0b01_000_010, 0b01_000_011, 0b01_000_100, 0b10_000_101, 0b10_000_110,
    0b11_000_111,
    0b11_001_111, 0b11_010_111, 0b11_011_111, 0b11_100_111, 0b11_101_111, 0b11_110_111,
];

static ALL_PAL8: sync::LazyLock<Vec<u8>> = sync::LazyLock::new(|| {
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
});

pub struct ColorPickerPopupWidget {
    pub close: bool,
    egui_id: egui::Id,
    close_on_pick: bool,
}

impl ColorPickerPopupWidget {
    pub fn new(egui_id: egui::Id, close_on_pick: bool) -> Self{
        ColorPickerPopupWidget {
            egui_id,
            close_on_pick,
            close: false,
        }
    }

    fn draw_palette(painter: &egui::Painter, rect: Rect, dims: (i32, i32), palette: &[u8], sel_color: Option<u8>) {
        let item_size = Vec2::new(rect.width() / (dims.0 as f32), rect.height() / (dims.1 as f32));
        for y in 0..dims.1 {
            for x in 0..dims.0 {
                let item_rect = Rect::from_min_size(rect.min + item_size * Vec2::new(x as f32, y as f32), item_size);
                let color = palette[(y*dims.0+x) as usize];
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
                if let Some(sel_color) = sel_color && sel_color == color {
                    let stroke = egui::Stroke::new(1.0, color_to_rgb_contrast(color));
                    painter.rect_stroke(item_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
                }
            }
        }
    }

    pub fn show(&mut self, response: &egui::Response, settings: &AppSettings, pick_color: &mut u8) {
        self.close = false;
        egui::containers::Popup::menu(response)
            .id(self.egui_id)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                // RGB values
                ui.horizontal(|ui| {
                    // R, G, B
                    let mut r = *pick_color & 7;
                    let mut g = (*pick_color >> 3) & 7;
                    let mut b = (*pick_color >> 6) & 3;
                    ui.add(egui::DragValue::new(&mut r).prefix("R ").speed(0.07).range(0..=7));
                    ui.add(egui::DragValue::new(&mut g).prefix("G ").speed(0.07).range(0..=7));
                    ui.add(egui::DragValue::new(&mut b).prefix("B ").speed(0.07).range(0..=3));
                    *pick_color = r | (g << 3) | (b << 6);

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // binary
                    ui.add(egui::DragValue::new(&mut *pick_color)
                           .prefix("binary: ")
                           .custom_formatter(move |n, _|
                                             format!("{:02b}_{:03b}_{:03b}",
                                                     n as i64 >> 6,
                                                     (n as i64 >> 3) & 0x7,
                                                     (n as i64) & 0x7))
                           .custom_parser(|s| i64::from_str_radix(&s.replace("_", ""), 2).map(|n| n as f64).ok())
                           .range(0..=255)
                    );

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // hex
                    ui.add(egui::DragValue::new(pick_color).prefix("hex: ").hexadecimal(2, true, true).range(0..=255));
                });

                // full palette
                ui.horizontal(|ui| {
                    let size = Vec2::new(32.0 * COLOR_BLOCK_SIZE, 8.0 * COLOR_BLOCK_SIZE);
                    let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
                    Self::draw_palette(&painter, response.rect, (32, 8), &ALL_PAL8, Some(*pick_color));
                    if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                        let x = ((pos.x - response.rect.min.x) / COLOR_BLOCK_SIZE).floor() as i32;
                        let y = ((pos.y - response.rect.min.y) / COLOR_BLOCK_SIZE).floor() as i32;
                        let index = y * 32 + x;
                        if index >= 0 && index <= ALL_PAL8.len() as i32 {
                            *pick_color = ALL_PAL8[index as usize];
                        }
                    }
                    if (self.close_on_pick && response.clicked()) || response.double_clicked() {
                        self.close = true;
                    }
                });

                ui.add_space(8.0);

                egui::Frame::NONE
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(4.0)
                    .fill(settings.color_picker_bg_color)
                    .show(ui, |ui| {
                        // grays palette
                        ui.horizontal(|ui| {
                            let num_colors = GRAY_PAL8.len() as i32;
                            let h_color_block_size = COLOR_BLOCK_SIZE * 13.0 / (num_colors as f32);
                            let alloc_size = Vec2::new(32.0 * COLOR_BLOCK_SIZE - 6.0, COLOR_BLOCK_SIZE);
                            let (response, painter) = ui.allocate_painter(alloc_size, Sense::click_and_drag());
                            let pal_width = h_color_block_size * num_colors as f32;
                            let draw_rect = egui::Rect::from_min_max(
                                egui::Pos2::new(response.rect.center().x - pal_width/2.0, response.rect.min.y),
                                egui::Pos2::new(response.rect.center().x + pal_width/2.0, response.rect.max.y)
                            );
                            Self::draw_palette(&painter, draw_rect, (num_colors, 1), &GRAY_PAL8, Some(*pick_color));
                            if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                                let index = ((pos.x - draw_rect.min.x) / h_color_block_size).floor() as i32;
                                if index >= 0 && index < num_colors {
                                    *pick_color = GRAY_PAL8[index as usize];
                                }
                            }
                            if (self.close_on_pick && response.clicked()) || response.double_clicked() {
                                self.close = true;
                            }
                        });

                        ui.add_space(2.0);

                        // gradients palette
                        ui.horizontal(|ui| {
                            let alloc_size = Vec2::new(32.0 * COLOR_BLOCK_SIZE - 6.0, 6.0 * COLOR_BLOCK_SIZE);
                            let (response, painter) = ui.allocate_painter(alloc_size, Sense::click_and_drag());
                            let pal_width = COLOR_BLOCK_SIZE * 13.0;
                            let draw_rect = egui::Rect::from_min_max(
                                egui::Pos2::new(response.rect.center().x - pal_width/2.0, response.rect.min.y),
                                egui::Pos2::new(response.rect.center().x + pal_width/2.0, response.rect.max.y)
                            );
                            Self::draw_palette(&painter, draw_rect, (13, 6), &GRAD_PAL8, Some(*pick_color));
                            if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                                let x = ((pos.x - draw_rect.min.x) / COLOR_BLOCK_SIZE).floor() as i32;
                                let y = ((pos.y - draw_rect.min.y) / COLOR_BLOCK_SIZE).floor() as i32;
                                let index = y * 13 + x;
                                if (0..13).contains(&x) && y >= 0 && index >= 0 && index < GRAD_PAL8.len() as i32 {
                                    *pick_color = GRAD_PAL8[index as usize];
                                }
                            }
                            if (self.close_on_pick && response.clicked()) || response.double_clicked() {
                                self.close = true;
                            }
                        });
                    });
            });
    }
}
