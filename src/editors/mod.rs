mod tileset;
mod map_data;
mod room;
mod sprite;
mod sprite_animation;
mod sfx;
mod mod_data;
mod font;
mod prop_font;
mod widgets;

pub use tileset::TilesetEditor;
pub use map_data::MapDataEditor;
pub use room::{RoomEditor, RoomEditorAssetLists};
pub use sprite::SpriteEditor;
pub use sprite_animation::SpriteAnimationEditor;
pub use sfx::SfxEditor;
pub use mod_data::ModDataEditor;
pub use font::FontEditor;
pub use prop_font::PropFontEditor;

use crate::data_asset::{DataAssetId, MapData, ImageCollectionAsset};
use crate::image::ImagePixels;
use egui::{Pos2, Rect};

pub struct DataAssetEditor {
    pub id: DataAssetId,
    pub egui_id: egui::Id,
    pub open: bool,
}

impl DataAssetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        DataAssetEditor {
            egui_id: egui::Id::new(format!("editor_{}", id)),
            id,
            open,
        }
    }

    pub fn calc_image_editor_window_size(image: &impl ImageCollectionAsset) -> (egui::Vec2, egui::Vec2) {
        let img_w = image.width() as f32;
        let img_h = image.height() as f32;
        let min_width = 130.0 + img_w + 220.0;
        let min_height = 2.0 * img_h + 80.0;
        let min_size = egui::Vec2::new(min_width, min_height);
        let default_size = egui::Vec2::new(min_width + 5.0 * img_w, min_height + 200.0).max(egui::Vec2::new(min_width + 200.0, 0.0));
        (min_size, default_size)
    }

    pub fn create_window<'a>(&'a mut self, wc: &crate::app::WindowContext, title: &str) -> egui::Window<'a> {
        let default_pos = wc.window_space.min + egui::Vec2::splat(10.0);
        let default_rect = egui::Rect {
            min: default_pos,
            max: default_pos + egui::Vec2::new(400.0, 240.0),
        };
        let frame = egui::Frame::window(&wc.egui.ctx.style()).inner_margin(1.0);
        egui::Window::new(title)
            .id(self.egui_id)
            .frame(frame)
            .enabled(! wc.sys_dialogs.has_open_dialog())
            .default_rect(default_rect)
            .max_size(wc.window_space.size())
            .constrain_to(wc.window_space)
            .open(&mut self.open)
    }
}

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
    pub fn from_rect(rect: Rect, map_data: &MapData, layer: MapLayer) -> Option<Self> {
        let (map_width, map_height) = match layer {
            MapLayer::Background => (map_data.bg_width, map_data.bg_height),
            _ => (map_data.width, map_data.height),
        };
        let rect = rect.intersect(Rect::from_min_max(Pos2::ZERO, Pos2::new(map_width as f32, map_height as f32)));
        Some(MapRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        })
    }
}

#[derive(Clone)]
pub struct MapLayerFragment {
    pub width: u32,
    pub height: u32,
    pub layer: MapLayer,
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
            layer: layer,
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

#[derive(Clone)]
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

pub enum ClipboardData {
    Image(ImagePixels),
    MapLayerFragment(MapLayerFragment),
    MapFullFragment(MapFullFragment),
}
