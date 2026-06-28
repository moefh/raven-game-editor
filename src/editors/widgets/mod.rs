mod color_picker;
mod color_picker_popup;
mod pal_color_picker;
mod map_editor;
mod sfx_editor;
mod sprite_frame_list_view;
mod room_editor;
mod world_editor;
mod world_region_editor;
mod map_view;
mod room_view;
mod image_editor;
mod image_picker;
mod prop_font_editor;
mod font_view;

use crate::data_asset::{
    MapData,
    World,
    WorldRegion,
};
use crate::app::AppSettings;
use crate::misc::current_time_as_millis;
use super::MapLayer;

use egui::{Vec2, Rect, Pos2};

pub const FULL_UV: Rect = Rect { min: Pos2::ZERO, max: Pos2::new(1.0, 1.0) };
pub const TILE_SIZE: f32 = crate::data_asset::Tileset::TILE_SIZE as f32;
pub const SCREEN_SIZE: Vec2 = Vec2::new(320.0, 240.0);

pub use color_picker::{*};
pub use color_picker_popup::{*};
pub use pal_color_picker::{*};
pub use map_editor::{*};
pub use sfx_editor::{*};
pub use sprite_frame_list_view::{*};
pub use room_editor::{*};
pub use world_editor::{*};
pub use world_region_editor::{*};
pub use map_view::{*};
pub use room_view::{*};
pub use image_editor::{*};
pub use image_picker::{*};
pub use prop_font_editor::{*};
pub use font_view::{*};

pub fn paint_marching_ants(painter: &egui::Painter, rect: egui::Rect, settings: &AppSettings) {
    let delay = settings.marching_ants_delay.max(10) as u64;
    let t = ((current_time_as_millis() / delay) & (i32::MAX as u64)) as i32;
    paint_ants(painter, rect, settings, t);
}

pub fn paint_ants(painter: &egui::Painter, rect: egui::Rect, settings: &AppSettings, t: i32) {
    let stroke1 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color1);
    let stroke2 = egui::Stroke::new(settings.marching_ants_thickness as f32, settings.marching_ants_color2);
    let dash_size = settings.marching_ants_dash_size.clamp(2, 16) as i32;

    let rect = rect.expand2(Vec2::splat(1.5));
    painter.rect_stroke(rect, egui::CornerRadius::ZERO, stroke1, egui::StrokeKind::Middle);

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

fn get_map_layer_tile(map_data: &MapData, layer: MapLayer, x: u32, y: u32) -> u8 {
    if matches!(layer, MapLayer::Parallax) && (x >= map_data.para_width || y >= map_data.para_height) { return MapData::NO_TILE; }
    if x >= map_data.width || y >= map_data.height { return MapData::NO_TILE; }

    match layer {
        MapLayer::Foreground => map_data.fg_tiles[(map_data.width * y + x) as usize],
        MapLayer::Background => map_data.bg_tiles[(map_data.width * y + x) as usize],
        MapLayer::Effects    => map_data.fx_tiles[(map_data.width * y + x) as usize],
        MapLayer::Parallax   => map_data.para_tiles[(map_data.para_width * y + x) as usize],
        _ => MapData::NO_TILE,
    }
}

struct WorldBorders {
    pub width: i32,
    pub height: i32,
    pub world_hash: u64,
    pub borders: Vec<u8>,
}

impl WorldBorders {
    pub const BORDER_TOP: u8 = 1 << 0;
    pub const BORDER_LEFT: u8 = 1 << 1;

    pub fn new() -> Self {
        WorldBorders {
            width: 0,
            height: 0,
            world_hash: 0,
            borders: Vec::new(),
        }
    }

    pub fn update(&mut self, world: &World) {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        world.hash(&mut hasher);
        let hash = hasher.finish();
        if self.world_hash != hash {
            self.world_hash = hash;
            self.calc_borders(&world);
        }
    }

    pub fn get_block_borders(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            0
        } else {
            self.borders[(y * self.width + x) as usize]
        }
    }

    fn get_world_block(x: i32, y: i32, world: &World) -> Option<u32> {
        for (region_index, region) in world.regions.iter().enumerate() {
            let bx = x - region.x as i32;
            let by = y - region.y as i32;
            let rw = region.width as i32;
            let rh = region.height as i32;
            if bx >= 0 && by >= 0 && bx < rw && by < rh {
                return region.blocks[(by * WorldRegion::BLOCK_STRIDE as i32 + bx) as usize].map(|block| {
                    block as u32 | ((region_index as u32) << 8)
                });
            }
        }
        None
    }

    fn calc_borders(&mut self, world: &World) {
        let (world_width, world_height) = world.regions.iter().fold((0, 0), |size, region| {
            (
                size.0.max(region.x as i32 + region.width as i32),
                size.1.max(region.y as i32 + region.height as i32),
            )
        });
        self.width = world_width + 1;
        self.height = world_height + 1;
        if self.borders.len() < (self.width * self.height) as usize {
            self.borders.resize((self.width * self.height) as usize, 0);
        }
        self.borders[..].fill(0);
        for y in 0..self.height {
            for x in 0..self.width {
                let block = Self::get_world_block(x, y, world);
                let left = if block != Self::get_world_block(x-1, y, world) { Self::BORDER_LEFT } else { 0 };
                let top  = if block != Self::get_world_block(x, y-1, world) { Self::BORDER_TOP  } else { 0 };
                self.borders[(y*self.width + x) as usize] = left | top;
            }
        }
    }
}
