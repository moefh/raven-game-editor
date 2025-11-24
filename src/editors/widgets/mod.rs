mod color_picker;
mod map_editor;
mod sfx_display;
mod sprite_frame_list_view;
mod room_editor;
mod map_view;
mod image_editor;
mod image_picker;

use crate::misc::ImageCollection;
use crate::data_asset::MapData;
use crate::app::AppSettings;

use egui::{Vec2, Sense, Image, Rect, Pos2, emath};

pub const FULL_UV : Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };
pub const TILE_SIZE: f32 = crate::data_asset::Tileset::TILE_SIZE as f32;

pub use color_picker::{*};
pub use map_editor::{*};
pub use sfx_display::{*};
pub use sprite_frame_list_view::{*};
pub use room_editor::{*};
pub use map_view::{*};
pub use image_editor::{*};
pub use image_picker::{*};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum MapLayer {
    Foreground,
    Clip,
    Effects,
    Background,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum RectBorder {
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
}

impl RectBorder {
    pub fn cursor(&self) -> egui::CursorIcon {
        match self {
            RectBorder::Left | RectBorder::Right => egui::CursorIcon::ResizeHorizontal,
            RectBorder::Top | RectBorder::Bottom => egui::CursorIcon::ResizeVertical,
            RectBorder::TopLeft | RectBorder::BottomRight => egui::CursorIcon::ResizeNwSe,
            RectBorder::TopRight | RectBorder::BottomLeft => egui::CursorIcon::ResizeNeSw,
        }
    }
}

pub fn paint_marching_ants(painter: &egui::Painter, rect: egui::Rect, settings: &AppSettings) {
    let stroke1 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color1);
    let stroke2 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color2);
    let delay = settings.marching_ants_delay.max(10) as u64;
    let dash_size = settings.marching_ants_dash_size.clamp(2, 16) as i32;

    let rect = rect.expand2(Vec2::splat(1.5));
    painter.rect_stroke(rect, egui::CornerRadius::ZERO, stroke1, egui::StrokeKind::Middle);

    let t = ((super::current_time_as_millis() / delay) & (i32::MAX as u64)) as i32;
    let n = t % (2 * dash_size) - dash_size;

    let width = rect.width().floor();
    if width >= 0.0 {
        let width = width as u32;
        for i in 0..width.div_ceil(2 * dash_size as u32) as i32 + 2 {
            let top_end = i * 2 * dash_size + n;
            if top_end > 0 {
                let x_start = (rect.min.x + (top_end - dash_size) as f32).clamp(rect.min.x, rect.max.x);
                let x_end   = (rect.min.x + top_end as f32).min(rect.max.x);
                painter.hline(x_start..=x_end, rect.min.y, stroke2);
            }

            let bot_end = i * 2 * dash_size - n;
            if bot_end > 0 {
                let x_start = (rect.min.x + (bot_end - dash_size) as f32).clamp(rect.min.x, rect.max.x);
                let x_end   = (rect.min.x + bot_end as f32).min(rect.max.x);
                painter.hline(x_start..=x_end, rect.max.y, stroke2);
            }
        }
    }

    let height = rect.height().floor();
    if height >= 0.0 {
        let height = height as u32;
        for i in 0..height.div_ceil(2 * dash_size as u32) as i32 + 2 {
            let left_end = i * 2 * dash_size - n;
            if left_end > 0 {
                let y_start = (rect.min.y + (left_end - dash_size) as f32).clamp(rect.min.y, rect.max.y);
                let y_end   = (rect.min.y + left_end as f32).min(rect.max.y);
                painter.vline(rect.min.x, y_start..=y_end, stroke2);
            }

            let right_end = i * 2 * dash_size + n;
            if right_end > 0 {
                let y_start = (rect.min.y + (right_end - dash_size) as f32).clamp(rect.min.y, rect.max.y);
                let y_end   = (rect.min.y + right_end as f32).min(rect.max.y);
                painter.vline(rect.max.x, y_start..=y_end, stroke2);
            }
        }
    }
}

fn get_map_layer_tile(map_data: &MapData, layer: MapLayer, x: u32, y: u32) -> u32 {
    if matches!(layer, MapLayer::Background) && (x >= map_data.bg_width || y >= map_data.bg_height) { return 0xff; }
    if x >= map_data.width || y >= map_data.height { return 0xff; }

    match layer {
        MapLayer::Foreground => map_data.fg_tiles[(map_data.width * y + x) as usize] as u32,
        MapLayer::Clip => map_data.clip_tiles[(map_data.width * y + x) as usize] as u32,
        MapLayer::Effects => map_data.fx_tiles[(map_data.width * y + x) as usize] as u32,
        MapLayer::Background => map_data.bg_tiles[(map_data.bg_width * y + x) as usize] as u32,
    }
}

pub fn prop_font_image_editor(ui: &mut egui::Ui, texture: &egui::TextureHandle, image: &ImageCollection,
                              selected_image: u32, image_width: u32) -> (egui::Response, emath::RectTransform) {
    let image_size = Vec2::new(image_width as f32, image.height as f32);
    let min_size = Vec2::splat(100.0).min(image_size + Vec2::splat(10.0)).max(ui.available_size());
    let (resp, painter) = ui.allocate_painter(min_size, Sense::drag());

    let resp_size = resp.rect.size();
    let (zoomx, zoomy) = (resp_size.x / (image_size.x + 1.0), (resp_size.y / (image_size.y + 1.0)));
    let image_zoom = f32::max(f32::min(zoomx, zoomy).floor(), 1.0);
    let center = resp.rect.center();
    let canvas_rect = Rect {
        min: center - image_zoom * image_size / 2.0,
        max: center + image_zoom * image_size / 2.0,
    };

    // draw background
    painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0xe0u8, 0xffu8, 0xffu8));

    // draw image
    let item_uv = Rect {
        min: Pos2::new(0.0, selected_image as f32 / image.num_items as f32),
        max: Pos2::new(image_width as f32 / image.width as f32, (selected_image+1) as f32 / image.num_items as f32),
    };
    Image::from_texture((texture.id(), image_size)).uv(item_uv).paint_at(ui, canvas_rect);

    // draw border
    let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    painter.rect_stroke(canvas_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

    // draw grid
    let canvas_size = canvas_rect.size();
    let display_grid = f32::min(canvas_size.x, canvas_size.y) / f32::max(image_size.x, image_size.y) > 3.0;
    if display_grid {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(112, 112, 112));
        for y in 0..=image.height {
            let py = canvas_rect.min.y + canvas_rect.height() * y as f32 / image.height as f32;
            painter.hline(canvas_rect.x_range(), py, stroke);
        }
        for x in 0..=image_width {
            let px = canvas_rect.min.x + canvas_rect.width() * x as f32 / image_width as f32;
            painter.vline(px, canvas_rect.y_range(), stroke);
        }
    }
    let canvas_to_image = emath::RectTransform::from_to(
        canvas_rect,
        Rect { min: Pos2::ZERO, max: Pos2::ZERO + image_size }
    );
    (resp, canvas_to_image)
}
