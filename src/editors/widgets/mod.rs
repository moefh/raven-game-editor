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

#[derive(Clone, Copy, PartialEq)]
pub enum MapLayer {
    Foreground,
    Clip,
    Effects,
    Background,
    Screen,
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

#[derive(Clone, Copy)]
pub struct MapRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl MapRect {
    //pub const ZERO: MapRect = MapRect { x: 0, y: 0, width: 0, height: 0 };

    pub fn from_rect(rect: Rect, map_data: &MapData, layer: MapLayer) -> Option<Self> {
        let (map_width, map_height) = match layer {
            MapLayer::Foreground | MapLayer::Clip | MapLayer::Effects => (map_data.width, map_data.height),
            MapLayer::Background => (map_data.bg_width, map_data.bg_height),
            _ => { return None; }
        };
        let rect = rect.intersect(Rect::from_min_max(Pos2::ZERO, Pos2::new(map_width as f32, map_height as f32)));
        Some(MapRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        })
    }

    //pub fn contains(&self, x: u32, y: u32) -> bool {
    //    x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    //}
}

pub struct MapLayerFragment {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl MapLayerFragment {
    pub fn copy_map(map_data: &mut MapData, layer: MapLayer, rect: MapRect) -> Option<MapLayerFragment> {
        let (map_width, map_height, map_data) = match layer {
            MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles),
            MapLayer::Clip       => (map_data.width, map_data.height, &mut map_data.clip_tiles),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
            MapLayer::Background => (map_data.bg_width, map_data.bg_height, &mut map_data.bg_tiles),
            _ => { return None; }
        };
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > map_width || rect.y + rect.height > map_height { return None; }

        let mut data = Vec::with_capacity((rect.width * rect.height) as usize);
        for y in 0..rect.height {
            for x in 0..rect.width {
                data.push(map_data[((rect.y + y) * map_width + rect.x + x) as usize]);
            }
        }
        Some(MapLayerFragment {
            width: rect.width,
            height: rect.height,
            data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, layer: MapLayer, rect: MapRect, fill_tile: u8) -> Option<MapLayerFragment> {
        let frag = Self::copy_map(map_data, layer, rect);
        if frag.is_some() {
            let (map_width, map_height, map_data) = match layer {
                MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles),
                MapLayer::Clip       => (map_data.width, map_data.height, &mut map_data.clip_tiles),
                MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
                MapLayer::Background => (map_data.bg_width, map_data.bg_height, &mut map_data.bg_tiles),
                _ => { return None; }
            };
            if rect.width > 0 && rect.height > 0 && rect.x + rect.width <= map_width && rect.y + rect.height <= map_height {
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        map_data[((rect.y + y) * map_width + rect.x + x) as usize] = fill_tile;
                    }
                }
            }
        }
        frag
    }

    pub fn paste_in_map(&self, x: i32, y: i32, map_data: &mut MapData, layer: MapLayer) {
        let (map_width, map_height, map_data, transparent) = match layer {
            MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles, true),
            MapLayer::Clip       => (map_data.width, map_data.height, &mut map_data.clip_tiles, true),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles, true),
            MapLayer::Background => (map_data.bg_width, map_data.bg_height, &mut map_data.bg_tiles, false),
            _ => { return; }
        };

        if (x > 0 &&   x  as u32 >= map_width) || (y > 0 &&   y  as u32 >= map_height) { return; }
        if (x < 0 && (-x) as u32 >= map_width) || (y < 0 && (-y) as u32 >= map_height) { return; }

        let mut src_x = 0;
        let mut src_y = 0;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = x;
        let mut y = y;
        if x < 0 { src_x = (-x) as u32; width -= src_x; x = 0; }
        if y < 0 { src_y = (-y) as u32; height -= src_y; y = 0; }
        let x = x as u32;
        let y = y as u32;
        if width > map_width - x { width = map_width - x; }
        if height > map_height - y { height = map_height - y; }

        for iy in 0..height {
            let src = ((iy + src_y) * self.width + src_x) as usize;
            let dest = ((iy + y) * map_width + x) as usize;
            if transparent {
                for ix in 0..width as usize {
                    let tile = self.data[src+ix];
                    if tile != 0xff {
                        map_data[dest+ix] = tile;
                    }
                }
            } else {
                map_data[dest .. dest + width as usize].clone_from_slice(&self.data[src .. src + width as usize]);
            }
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> u8 {
        if x > self.width || y > self.height { return 0xff; }
        self.data[(y * self.width + x) as usize]
    }
}

pub struct MapFullFragment {
    pub width: u32,
    pub height: u32,
    pub fg_data: Vec<u8>,
    pub clip_data: Vec<u8>,
    pub fx_data: Vec<u8>,
    pub bg_data: Vec<u8>,
}

impl MapFullFragment {
    pub fn copy_map(map_data: &mut MapData, rect: MapRect, include_bg: bool) -> Option<MapFullFragment> {
        let map_width = map_data.width;
        let map_height = map_data.height;
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > map_width || rect.y + rect.height > map_height { return None; }

        let copy_bg = include_bg && map_data.bg_width == map_data.width && map_data.bg_height == map_data.height;

        let num_tiles = (rect.width * rect.height) as usize;
        let mut fg_data = Vec::with_capacity(num_tiles);
        let mut clip_data = Vec::with_capacity(num_tiles);
        let mut fx_data = Vec::with_capacity(num_tiles);
        let mut bg_data = if copy_bg { Vec::with_capacity(num_tiles) } else { Vec::new() };
        for y in 0..rect.height {
            for x in 0..rect.width {
                let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                fg_data.push(map_data.fg_tiles[tile_index]);
                clip_data.push(map_data.clip_tiles[tile_index]);
                fx_data.push(map_data.fx_tiles[tile_index]);
                if copy_bg {
                    bg_data.push(map_data.bg_tiles[tile_index]);
                }
            }
        }
        Some(MapFullFragment {
            width: rect.width,
            height: rect.height,
            fg_data,
            clip_data,
            fx_data,
            bg_data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, rect: MapRect, fill_tile: u8, include_bg: bool) -> Option<MapFullFragment> {
        let frag = Self::copy_map(map_data, rect, include_bg);
        if frag.is_some() {
            let map_width = map_data.width;
            let map_height = map_data.height;
            if rect.width > 0 && rect.height > 0 && rect.x + rect.width <= map_width && rect.y + rect.height <= map_height {
                let erase_bg = include_bg && map_data.bg_width == map_data.width && map_data.bg_height == map_data.height;
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                        map_data.fg_tiles[tile_index] = 0xff;
                        map_data.clip_tiles[tile_index] = 0xff;
                        map_data.fx_tiles[tile_index] = 0xff;
                        if erase_bg {
                            map_data.bg_tiles[tile_index] = fill_tile;
                        }
                    }
                }
            }
        }
        frag
    }

    pub fn paste_in_map(&self, x: i32, y: i32, map_data: &mut MapData) {
        let map_width = map_data.width;
        let map_height = map_data.height;
        if (x > 0 &&   x  as u32 >= map_width) || (y > 0 &&   y  as u32 >= map_height) { return; }
        if (x < 0 && (-x) as u32 >= map_width) || (y < 0 && (-y) as u32 >= map_height) { return; }

        let mut src_x = 0;
        let mut src_y = 0;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = x;
        let mut y = y;
        if x < 0 { src_x = (-x) as u32; width -= src_x; x = 0; }
        if y < 0 { src_y = (-y) as u32; height -= src_y; y = 0; }
        let x = x as u32;
        let y = y as u32;
        if width > map_width - x { width = map_width - x; }
        if height > map_height - y { height = map_height - y; }

        for iy in 0..height {
            let src = ((iy + src_y) * self.width + src_x) as usize;
            let dest = ((iy + y) * map_width + x) as usize;
            for ix in 0..width as usize {
                let fg_tile = self.fg_data[src+ix];     if fg_tile   != 0xff { map_data.fg_tiles[dest+ix] = fg_tile; }
                let clip_tile = self.clip_data[src+ix]; if clip_tile != 0xff { map_data.clip_tiles[dest+ix] = clip_tile; }
                let fx_tile = self.fx_data[src+ix];     if fx_tile   != 0xff { map_data.fx_tiles[dest+ix] = fx_tile; }
                if ! self.bg_data.is_empty() {
                    map_data.bg_tiles[dest+ix] = self.bg_data[src+ix];
                }
            }
        }
    }

    pub fn get_layer_tile(&self, x: u32, y: u32, layer: MapLayer) -> u8 {
        if x > self.width || y > self.height { return 0xff; }
        match layer {
            MapLayer::Foreground => {
                self.fg_data[(y * self.width + x) as usize]
            }
            MapLayer::Clip => {
                self.clip_data[(y * self.width + x) as usize]
            }
            MapLayer::Effects => {
                self.fx_data[(y * self.width + x) as usize]
            }
            MapLayer::Background => {
                if self.bg_data.is_empty() { return 0xff; }
                self.bg_data[(y * self.width + x) as usize]
            }
            _ => 0xff
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
        _ => 0xff,
    }
}
