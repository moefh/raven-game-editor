mod color_picker;
mod map_editor;
mod sfx_editor;
mod sprite_frame_list_view;
mod room_editor;
mod map_view;
mod image_editor;
mod image_picker;
mod prop_font_editor;
mod font_view;

use crate::data_asset::MapData;
use crate::app::AppSettings;
use crate::misc::current_time_as_millis;
use super::MapLayer;

use egui::{Vec2, Rect, Pos2};

pub const FULL_UV: Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };
pub const TILE_SIZE: f32 = crate::data_asset::Tileset::TILE_SIZE as f32;
pub const SCREEN_SIZE: Vec2 = Vec2::new(320.0, 240.0);

pub use color_picker::{*};
pub use map_editor::{*};
pub use sfx_editor::{*};
pub use sprite_frame_list_view::{*};
pub use room_editor::{*};
pub use map_view::{*};
pub use image_editor::{*};
pub use image_picker::{*};
pub use prop_font_editor::{*};
pub use font_view::{*};

pub fn paint_marching_ants(painter: &egui::Painter, rect: egui::Rect, settings: &AppSettings) {
    let stroke1 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color1);
    let stroke2 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color2);
    let delay = settings.marching_ants_delay.max(10) as u64;
    let dash_size = settings.marching_ants_dash_size.clamp(2, 16) as i32;

    let rect = rect.expand2(Vec2::splat(1.5));
    painter.rect_stroke(rect, egui::CornerRadius::ZERO, stroke1, egui::StrokeKind::Middle);

    let t = ((current_time_as_millis() / delay) & (i32::MAX as u64)) as i32;
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
        _ => 0xff,
    }
}
