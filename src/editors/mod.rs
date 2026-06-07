mod tileset;
mod map_data;
mod room;
mod sprite;
mod pal_sprite;
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
pub use pal_sprite::PalSpriteEditor;
pub use sprite_animation::SpriteAnimationEditor;
pub use sfx::SfxEditor;
pub use mod_data::ModDataEditor;
pub use font::FontEditor;
pub use prop_font::PropFontEditor;

pub use widgets::ColorPickerPopupWidget;

use crate::include_ref_image;
use crate::misc::{calc_hash, get_asset_type_image, ImageRef, IMAGES};
use crate::data_asset::{
    DataAssetId,
    AssetIdCollection,
    MapData,
    GenericAsset,
    RoomTriggerType,
};
use crate::image::{ImagePixels, ImageCollection, ImageSlicingMethod};
use crate::app::WindowContext;
use egui::{Pos2, Rect};

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorWindowAction {
    None,
    Close,
    ToggleMaximize,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum MaximizedState {
    Maximized,
    UnmaxRequested,
    UnmaxSizeReset,
    Normal,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ImageSlicingMethodOption {
    BySize,
    ByNumber,
}

impl ImageSlicingMethodOption {
    pub fn from_slicing_method(method: &ImageSlicingMethod) -> Self {
        match method {
            ImageSlicingMethod::BySize{..} => ImageSlicingMethodOption::BySize,
            ImageSlicingMethod::ByNumber{..} => ImageSlicingMethodOption::ByNumber,
        }
    }
    pub fn text(&self) -> &str {
        match self {
            ImageSlicingMethodOption::BySize => "by size",
            ImageSlicingMethodOption::ByNumber => "by number",
        }
    }
}

pub struct AssetEditorTitle {
    image: ImageRef,
    title: String,
}

pub struct AssetEditorBase {
    pub id: DataAssetId,
    pub egui_id: egui::Id,
    pub open: bool,
    pub closed_last_frame: bool,
    maximized_state: MaximizedState,
    window_rect: egui::Rect,
    saved_hash: u64,
    cur_hash: u64,
}

impl AssetEditorBase {
    fn new(id: DataAssetId, open: bool) -> Self {
        AssetEditorBase {
            id,
            open,
            closed_last_frame: false,
            egui_id: egui::Id::new(format!("editor_{}", id)),
            maximized_state: MaximizedState::Normal,
            window_rect: egui::Rect::ZERO,
            saved_hash: 0,
            cur_hash: 0,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.cur_hash != self.saved_hash
    }

    pub fn clear_dirty(&mut self, asset: &impl std::hash::Hash) {
        self.cur_hash = calc_hash(asset);
        self.saved_hash = self.cur_hash;
    }

    pub fn update_dirty(&mut self, asset: &impl std::hash::Hash) {
        self.cur_hash = calc_hash(asset);
    }

    fn calc_image_editor_window_size(image: &impl ImageCollection) -> (egui::Vec2, egui::Vec2) {
        let img_w = image.width() as f32;
        let img_h = image.height() as f32;
        let min_width = img_w + 320.0;
        let min_height = img_h + 200.0;
        let min_size = egui::Vec2::new(min_width, min_height);
        let default_size = egui::Vec2::new(min_width + img_w * 0.5 + 100.0, min_height + img_h * 0.5 + 100.0);
        (min_size, default_size)
    }

    fn window_title(&self, asset: &impl GenericAsset) -> AssetEditorTitle {
        let title = if self.is_dirty() {
            format!("{} (modified)", asset.asset().name)
        } else {
            asset.asset().name.clone()
        };

        AssetEditorTitle {
            title,
            image: get_asset_type_image(asset.asset().asset_type),
        }
    }

    pub fn toggle_open(&mut self) {
        self.open = ! self.open;
        if ! self.open {
            self.closed_last_frame = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.closed_last_frame = true;
    }

    fn toggle_maximized(&mut self) {
        match self.maximized_state {
            MaximizedState::Maximized => {
                self.maximized_state = MaximizedState::UnmaxRequested;
            }
            MaximizedState::Normal => {
                self.maximized_state = MaximizedState::Maximized;
            }
            _ => {}
        }
    }

    fn show_window(&mut self, wc: &mut WindowContext, title: &AssetEditorTitle,
                   min_size: impl Into<egui::Vec2>, default_size: impl Into<egui::Vec2>,
                   show_fn: impl FnOnce(&mut egui::Ui, &mut WindowContext)) {
        let maximized_state = self.maximized_state;
        let resp = self.create_window(wc, &title.title, min_size, default_size).show(wc.egui.ctx, |ui| {
            let frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 1 });
            let action = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(3.0);
                    ui.add(egui::Button::image(include_ref_image!(title.image)).frame(false));
                    ui.add(egui::Label::new(&title.title).selectable(false));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut action = EditorWindowAction::None;

                        ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                            action = EditorWindowAction::Close;
                        }
                        let image = match maximized_state {
                            MaximizedState::Maximized => { egui::Image::new(IMAGES.un_maximize) }
                            _ => { egui::Image::new(IMAGES.maximize) }
                        };
                        if ui.add(egui::Button::image(image).frame_when_inactive(false)).clicked() {
                            action = EditorWindowAction::ToggleMaximize;
                        }

                        action
                    }).inner
                }).inner
            }).inner;

            show_fn(ui, wc);
            action
        });

        if let Some(resp) = resp {
            // close window if show() above returned true
            if let Some(action) = resp.inner {
                match action {
                    EditorWindowAction::None => {}
                    EditorWindowAction::Close => { self.close(); }
                    EditorWindowAction::ToggleMaximize => { self.toggle_maximized(); }
                }
            }

            // save window position/size if not maximized
            if matches!(self.maximized_state, MaximizedState::Normal) {
                self.window_rect = resp.response.rect;
            }

            // consume CTRL+UP to maximize/unmaximize
            if wc.is_editor_on_top(self.id) {
                let ctrl_up = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowUp);
                if resp.response.ctx.input_mut(|i| i.consume_shortcut(&ctrl_up)) {
                    self.toggle_maximized();
                }
            }
        }
    }

    fn create_window<'a>(&'a mut self, wc: &WindowContext, title: &str,
                         min_size: impl Into<egui::Vec2>, default_size: impl Into<egui::Vec2>) -> egui::Window<'a> {
        let default_pos = wc.window_space.min + egui::Vec2::splat(10.0);
        let default_rect = egui::Rect {
            min: default_pos,
            max: default_pos + default_size.into(),
        };
        if self.window_rect.max.x == self.window_rect.min.x || self.window_rect.max.y == self.window_rect.min.x {
            self.window_rect = default_rect;
        }

        let selected = wc.is_editor_on_top(self.id);
        let title_bg = match wc.egui.ctx.theme() {
            egui::Theme::Light => if selected {
                egui::Color32::from_rgb(0xe0, 0xe0, 0xe0)
            } else {
                wc.egui.ctx.global_style().visuals.window_fill
            },
            egui::Theme::Dark => if selected {
                egui::Color32::from_rgb(0, 0x20, 0x40)
            } else {
                egui::Color32::from_rgb(0, 0x10, 0x20)
            },
        };
        let mut frame = egui::Frame::window(&wc.egui.ctx.global_style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
            .fill(title_bg);
        if self.open && ! matches!(self.maximized_state, MaximizedState::Normal) {
            frame = frame.corner_radius(0.0);
            let (win_rect, constrain_rect) = match self.maximized_state {
                MaximizedState::Maximized => {
                    let win_rect = egui::Rect {
                        min: wc.window_space.min + egui::Vec2::new(-10.0, 0.0),
                        max: wc.window_space.max + egui::Vec2::new(-6.0, 0.0),
                    };
                    (win_rect, wc.window_space)
                }
                MaximizedState::UnmaxRequested => {
                    self.maximized_state = MaximizedState::UnmaxSizeReset;
                    let rect = self.window_rect;
                    (rect, rect.with_max_x(rect.max.x-2.0).with_max_y(rect.max.y-2.0))
                }
                _ => {
                    self.maximized_state = MaximizedState::Normal;
                    let rect = self.window_rect;
                    (rect, rect.with_max_x(rect.max.x-2.0).with_max_y(rect.max.y-2.0))
                }
            };
            egui::Window::new(title)
                .id(self.egui_id)
                .frame(frame)
                .enabled(! wc.sys_dialogs.has_open_dialog())
                .fixed_rect(win_rect)
                .constrain_to(constrain_rect)
                .collapsible(false)
                .title_bar(false)
                .open(&mut self.open)
        } else {
            egui::Window::new(title)
                .id(self.egui_id)
                .frame(frame)
                .enabled(! wc.sys_dialogs.has_open_dialog())
                .default_rect(default_rect)
                .min_size(min_size)
                .max_size(wc.window_space.size())
                .constrain_to(wc.window_space)
                .collapsible(false)
                .title_bar(false)
                .open(&mut self.open)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapLayer {
    Foreground,
    Background,
    Effects,
    Parallax,
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
            MapLayer::Parallax => (map_data.para_width, map_data.para_height),
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
            MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
            MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles),
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
            layer,
            data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, layer: MapLayer, rect: MapRect, fill_tile: u8) -> Option<MapLayerFragment> {
        let frag = Self::copy_map(map_data, layer, rect);
        if frag.is_some() {
            let (map_width, map_height, map_data) = match layer {
                MapLayer::Foreground => (map_data.width, map_data.height, &mut map_data.fg_tiles),
                MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles),
                MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles),
                MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles),
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
            MapLayer::Background => (map_data.width, map_data.height, &mut map_data.bg_tiles, true),
            MapLayer::Effects    => (map_data.width, map_data.height, &mut map_data.fx_tiles, true),
            MapLayer::Parallax   => (map_data.para_width, map_data.para_height, &mut map_data.para_tiles, false),
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
                    if tile != MapData::NO_TILE {
                        map_data[dest+ix] = tile;
                    }
                }
            } else {
                map_data[dest .. dest + width as usize].clone_from_slice(&self.data[src .. src + width as usize]);
            }
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> u8 {
        if x > self.width || y > self.height { return MapData::NO_TILE; }
        self.data[(y * self.width + x) as usize]
    }
}

pub struct MapUndoData {
    pub width: u32,
    pub height: u32,
    pub para_width: u32,
    pub para_height: u32,
    pub fg_tiles: Vec<u8>,
    pub bg_tiles: Vec<u8>,
    pub fx_tiles: Vec<u8>,
    pub para_tiles: Vec<u8>,
}

impl MapUndoData {
    pub fn from_map(map_data: &MapData) -> Self {
        let num_full_tiles = (map_data.width * map_data.height) as usize;
        let mut fg_tiles = Vec::with_capacity(num_full_tiles);
        let mut bg_tiles = Vec::with_capacity(num_full_tiles);
        let mut fx_tiles = Vec::with_capacity(num_full_tiles);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile_index = (map_data.width * y + x) as usize;
                fg_tiles.push(map_data.fg_tiles[tile_index]);
                bg_tiles.push(map_data.bg_tiles[tile_index]);
                fx_tiles.push(map_data.fx_tiles[tile_index]);
            }
        }
        let num_para_tiles = (map_data.para_width * map_data.para_height) as usize;
        let mut para_tiles = Vec::with_capacity(num_para_tiles);
        for y in 0..map_data.para_height {
            for x in 0..map_data.para_width {
                let tile_index = (map_data.para_width * y + x) as usize;
                para_tiles.push(map_data.para_tiles[tile_index]);
            }
        }
        MapUndoData {
            width: map_data.width,
            height: map_data.height,
            para_width: map_data.para_width,
            para_height: map_data.para_height,
            fg_tiles,
            bg_tiles,
            fx_tiles,
            para_tiles,
        }
    }

    pub fn to_map(&self, map_data: &mut MapData) -> bool {
        if self.width != map_data.width || self.height != map_data.height ||
            self.para_width != map_data.para_width || self.para_height != map_data.para_height {
                return false;
            }
        let map_width = map_data.width as usize;
        for y in 0..map_data.height {
            let tile_index = (map_data.width * y) as usize;
            map_data.fg_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.fg_tiles[tile_index..tile_index+map_width]);
            map_data.bg_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.bg_tiles[tile_index..tile_index+map_width]);
            map_data.fx_tiles[tile_index..tile_index+map_width].copy_from_slice(&self.fx_tiles[tile_index..tile_index+map_width]);
        }
        let map_para_width = map_data.para_width as usize;
        for y in 0..map_data.para_height {
            let tile_index = (map_data.para_width * y) as usize;
            map_data.para_tiles[tile_index..tile_index+map_para_width].copy_from_slice(&self.para_tiles[tile_index..tile_index+map_para_width]);
        }
        true
    }
}

#[derive(Clone)]
pub struct MapWholeFragment {
    pub width: u32,
    pub height: u32,
    pub fg_data: Vec<u8>,
    pub bg_data: Vec<u8>,
    pub fx_data: Vec<u8>,
    pub para_data: Vec<u8>,
}

impl MapWholeFragment {
    pub fn copy_map(map_data: &MapData, rect: MapRect, include_para: bool) -> Option<MapWholeFragment> {
        let map_width = map_data.width;
        let map_height = map_data.height;
        if rect.width == 0 || rect.height == 0 || rect.x + rect.width > map_width || rect.y + rect.height > map_height { return None; }

        let copy_para = include_para && map_data.para_width == map_data.width && map_data.para_height == map_data.height;

        let num_tiles = (rect.width * rect.height) as usize;
        let mut fg_data = Vec::with_capacity(num_tiles);
        let mut bg_data = Vec::with_capacity(num_tiles);
        let mut fx_data = Vec::with_capacity(num_tiles);
        let mut para_data = if copy_para { Vec::with_capacity(num_tiles) } else { Vec::new() };
        for y in 0..rect.height {
            for x in 0..rect.width {
                let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                fg_data.push(map_data.fg_tiles[tile_index]);
                bg_data.push(map_data.bg_tiles[tile_index]);
                fx_data.push(map_data.fx_tiles[tile_index]);
                if copy_para {
                    para_data.push(map_data.bg_tiles[tile_index]);
                }
            }
        }
        Some(MapWholeFragment {
            width: rect.width,
            height: rect.height,
            fg_data,
            bg_data,
            fx_data,
            para_data,
        })
    }

    pub fn cut_map(map_data: &mut MapData, rect: MapRect, fill_tile: u8, include_para: bool) -> Option<MapWholeFragment> {
        let frag = Self::copy_map(map_data, rect, include_para);
        if frag.is_some() {
            let map_width = map_data.width;
            let map_height = map_data.height;
            if rect.width > 0 && rect.height > 0 && rect.x + rect.width <= map_width && rect.y + rect.height <= map_height {
                let erase_para = include_para && map_data.para_width == map_data.width && map_data.para_height == map_data.height;
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        let tile_index = ((rect.y + y) * map_width + rect.x + x) as usize;
                        map_data.fg_tiles[tile_index] = MapData::NO_TILE;
                        map_data.bg_tiles[tile_index] = fill_tile;
                        map_data.fx_tiles[tile_index] = MapData::NO_TILE;
                        if erase_para {
                            map_data.para_tiles[tile_index] = fill_tile;
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
                let fg_tile = self.fg_data[src+ix]; if fg_tile != MapData::NO_TILE { map_data.fg_tiles[dest+ix] = fg_tile; }
                let bg_tile = self.bg_data[src+ix]; if bg_tile != MapData::NO_TILE { map_data.bg_tiles[dest+ix] = bg_tile; }
                let fx_tile = self.fx_data[src+ix]; if fx_tile != MapData::NO_TILE { map_data.fx_tiles[dest+ix] = fx_tile; }
                if ! self.para_data.is_empty() {
                    map_data.para_tiles[dest+ix] = self.para_data[src+ix];
                }
            }
        }
    }

    pub fn get_layer_tile(&self, x: u32, y: u32, layer: MapLayer) -> u8 {
        if x > self.width || y > self.height { return MapData::NO_TILE; }
        match layer {
            MapLayer::Foreground => { self.fg_data[(y * self.width + x) as usize] }
            MapLayer::Background => { self.bg_data[(y * self.width + x) as usize] }
            MapLayer::Effects    => { self.fx_data[(y * self.width + x) as usize] }
            MapLayer::Parallax => {
                if self.para_data.is_empty() { return MapData::NO_TILE; }
                self.para_data[(y * self.width + x) as usize]
            }
            _ => MapData::NO_TILE
        }
    }
}

pub enum ImageClipboardData {
    Empty,
    Image(ImagePixels),
}

impl ImageClipboardData {
    pub fn is_none(&self) -> bool {
        matches!(self, ImageClipboardData::Empty)
    }

    pub fn take(&mut self) -> ImageClipboardData {
        std::mem::replace(self, ImageClipboardData::Empty)
    }
}

pub enum MapClipboardData {
    Empty,
    MapLayerFragment(MapLayerFragment),
    MapWholeFragment(MapWholeFragment),
}

impl MapClipboardData {
    pub fn is_none(&self) -> bool {
        matches!(self, MapClipboardData::Empty)
    }

    pub fn take(&mut self) -> MapClipboardData {
        std::mem::replace(self, MapClipboardData::Empty)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoomTriggerTypeSel {
    Unknown,
    Door,
    Trap,
    PlayerSpawn,
    EnemySpawn,
}

impl RoomTriggerTypeSel {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown { .. } => RoomTriggerTypeSel::Unknown,
            RoomTriggerType::Door { .. } => RoomTriggerTypeSel::Door,
            RoomTriggerType::PlayerSpawn { .. } => RoomTriggerTypeSel::PlayerSpawn,
            RoomTriggerType::EnemySpawn { .. } => RoomTriggerTypeSel::EnemySpawn,
            RoomTriggerType::Trap { .. } => RoomTriggerTypeSel::Trap,
        }
    }

    pub fn convert_trigger_type(&self, trigger_type: &mut RoomTriggerType, asset_ids: &AssetIdCollection) {
        match self {
            RoomTriggerTypeSel::Unknown if ! matches!(trigger_type, RoomTriggerType::Unknown {..}) => {
                *trigger_type = RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0 };
            }
            RoomTriggerTypeSel::Trap if ! matches!(trigger_type, RoomTriggerType::Trap {..}) => {
                *trigger_type = RoomTriggerType::Trap { width: 64, height: 64, type_id: 0 };
            }
            RoomTriggerTypeSel::PlayerSpawn if ! matches!(trigger_type, RoomTriggerType::PlayerSpawn {..}) => {
                *trigger_type = RoomTriggerType::PlayerSpawn { direction: 0 };
            }
            RoomTriggerTypeSel::EnemySpawn if ! matches!(trigger_type, RoomTriggerType::EnemySpawn {..}) => {
                if let Some(animation_id) = asset_ids.animations.get_first() {
                    *trigger_type = RoomTriggerType::EnemySpawn { animation_id };
                }
            }
            RoomTriggerTypeSel::Door if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                if let Some(room_id) = asset_ids.rooms.get_first() {
                    *trigger_type = RoomTriggerType::Door { room_id, door_id: 0 };
                }
            }
            _ => {}
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            RoomTriggerTypeSel::Unknown => "any",
            RoomTriggerTypeSel::Door => "door",
            RoomTriggerTypeSel::Trap => "trap",
            RoomTriggerTypeSel::PlayerSpawn => "player spawn",
            RoomTriggerTypeSel::EnemySpawn => "enemy spawn",
        }
    }
}
