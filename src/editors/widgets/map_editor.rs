use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::{MapData, Tileset};
use crate::misc::STATIC_IMAGES;
use egui::{Vec2, Sense, Rect, Pos2, Color32, Image};
use egui::emath;

use crate::app::KeyboardPressed;
use super::{TILE_SIZE, SCREEN_SIZE, get_map_layer_tile};
use super::super::{MapClipboardData, MapUndoData, MapFullFragment, MapLayerFragment, MapRect, MapLayer};

pub enum MapSelection {
    None,
    Rect(Pos2, Pos2),
    LayerFragment(Pos2, MapLayerFragment),
    FullFragment(Pos2, MapFullFragment),
}

impl MapSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, MapSelection::LayerFragment(..) | MapSelection::FullFragment(..))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MapSelection::None => true,
            MapSelection::Rect(origin, end) => {
                let width = end.x - origin.x;
                let height = end.y - origin.y;
                width.abs() == 0.0 && height.abs() == 0.0
            }
            MapSelection::LayerFragment(_, _) => false,
            MapSelection::FullFragment(_, _) => false,
        }
    }

    pub fn take_layer_fragment(&mut self) -> Option<(Pos2, MapLayerFragment)> {
        match self {
            MapSelection::LayerFragment(..) => {
                let mut ret = MapSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    MapSelection::LayerFragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen, but it's not a big deal
                }
            }
            _ => None
        }
    }

    pub fn take_full_fragment(&mut self) -> Option<(Pos2, MapFullFragment)> {
        match self {
            MapSelection::FullFragment(..) => {
                let mut ret = MapSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    MapSelection::FullFragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen, but it's not a big deal
                }
            }
            _ => None
        }
    }

    pub fn get_rect(&self) -> Option<Rect> {
        match self {
            MapSelection::Rect(origin, end) => {
                Some(Rect {
                    min: Pos2::new(origin.x.min(end.x), origin.y.min(end.y)),
                    max: Pos2::new(origin.x.max(end.x), origin.y.max(end.y)),
                })
            }
            MapSelection::LayerFragment(pos, frag) => {
                Some(Rect {
                    min: *pos,
                    max: *pos + Vec2::new(frag.width as f32, frag.height as f32),
                })
            }
            MapSelection::FullFragment(pos, frag) => {
                Some(Rect {
                    min: *pos,
                    max: *pos + Vec2::new(frag.width as f32, frag.height as f32),
                })
            }
            MapSelection::None => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MapDisplay {
    bits: u8,
}

impl MapDisplay {
    pub const FOREGROUND: u8  = 1 << 0;
    pub const CLIP: u8        = 1 << 1;
    pub const EFFECTS: u8     = 1 << 2;
    pub const BACKGROUND: u8  = 1 << 3;
    pub const GRID: u8        = 1 << 4;
    pub const SCREEN: u8      = 1 << 5;

    pub fn new(bits: u8) -> Self {
        MapDisplay {
            bits,
        }
    }

    pub fn toggle(&mut self, bits: u8) {
        self.bits ^= bits;
    }

    pub fn set(&mut self, bits: u8) {
        self.bits |= bits;
    }

    pub fn has_bits(&self, bits: u8) -> bool {
        (self.bits & bits) != 0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapTool {
    Pencil,
    SelectLayer,
    SelectFgLayers,
    SelectAllLayers,
}

pub struct MapEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub edit_layer: MapLayer,
    pub display: MapDisplay,
    pub tool: MapTool,
    pub left_draw_tile: u32,
    pub right_draw_tile: u32,
    pub hover_pos: Vec2,
    pub custom_grid_color: Option<Color32>,
    pub screen_display_pos: Vec2,
    pub selection: MapSelection,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
    tool_changed: bool,
    edit_layer_changed: bool,
    undo_target: Option<MapUndoData>,
}

impl MapEditorWidget {
    pub fn new() -> Self {
        let zoom = 1.0;
        MapEditorWidget {
            zoom,
            scroll: Vec2::ZERO,
            edit_layer: MapLayer::Screen,
            tool: MapTool::Pencil,
            display: MapDisplay::new(MapDisplay::FOREGROUND | MapDisplay::BACKGROUND | MapDisplay::GRID),
            left_draw_tile: 0,
            right_draw_tile: 0xff,
            hover_pos: Vec2::ZERO,
            custom_grid_color: None,
            screen_display_pos: Vec2::splat(TILE_SIZE/zoom),
            selection: MapSelection::None,
            drag_mouse_origin: Pos2::ZERO,
            drag_frag_origin: Pos2::ZERO,
            tool_changed: false,
            edit_layer_changed: false,
            undo_target: None,
        }
    }

    pub fn set_undo_target(&mut self, map_data: &MapData) {
        self.undo_target = Some(MapUndoData::from_map(map_data));
    }

    pub fn set_edit_layer(&mut self, layer: MapLayer) {
        self.edit_layer = layer;
        self.edit_layer_changed = true;
    }

    pub fn set_tool(&mut self, tool: MapTool) {
        self.tool = tool;
        self.tool_changed = true;
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, map_data: &MapData) {
        let zoom = zoom.max(0.25);
        let map_size = Vec2 {
            x: map_data.width as f32 * TILE_SIZE * zoom,
            y: map_data.height as f32 * TILE_SIZE * zoom,
        };
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, map_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, map_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - map_size).min(Vec2::ZERO);
    }

    pub fn lift_selection(&mut self, map_data: &mut MapData) {
        if self.selection.is_floating() { return; } // already floating

        let fill_tile = self.get_fill_bg_tile_for_layer();
        if let Some(sel_rect) = self.selection.get_rect() &&
            sel_rect.is_positive() &&
            let Some(map_rect) = MapRect::from_rect(sel_rect, map_data, self.edit_layer) {
                self.selection = match self.tool {
                    MapTool::SelectLayer => {
                        self.set_undo_target(map_data);
                        if let Some(frag) = MapLayerFragment::cut_map(map_data, self.edit_layer, map_rect, fill_tile) {
                            MapSelection::LayerFragment(sel_rect.min, frag)
                        } else {
                            MapSelection::None
                        }
                    }

                    MapTool::SelectAllLayers | MapTool::SelectFgLayers => {
                        self.set_undo_target(map_data);
                        let include_bg_layer = self.tool == MapTool::SelectAllLayers;
                        if let Some(frag) = MapFullFragment::cut_map(map_data, map_rect, fill_tile, include_bg_layer) {
                            MapSelection::FullFragment(sel_rect.min, frag)
                        } else {
                            MapSelection::None
                        }
                    }

                    _ => MapSelection::None
                }
            }
    }

    pub fn delete_selection(&mut self, map_data: &mut MapData) {
        self.lift_selection(map_data);
        self.selection = MapSelection::None;
    }

    pub fn drop_selection(&mut self, map_data: &mut MapData) {
        match &self.selection {
            MapSelection::LayerFragment(pos, frag) => {
                frag.paste_in_map(pos.x as i32, pos.y as i32, map_data, self.edit_layer);
            }
            MapSelection::FullFragment(pos, frag) => {
                frag.paste_in_map(pos.x as i32, pos.y as i32, map_data);
            }
            _ => {}
        }
        self.selection = MapSelection::None;
    }

    fn get_selected_tile_for_click(&self, response: &egui::Response) -> Option<u32> {
        if response.dragged_by(egui::PointerButton::Primary) {
            Some(self.left_draw_tile)
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            Some(self.right_draw_tile)
        } else {
            None
        }
    }

    fn get_fill_bg_tile_for_layer(&self) -> u8 {
        match self.edit_layer {
            MapLayer::Foreground | MapLayer::Clip | MapLayer::Effects => 0xff,
            MapLayer::Background => self.right_draw_tile as u8,
            _ => 0xff,
        }
    }

    fn paint_floating_selection_for_layer(&self, ui: &mut egui::Ui, layer: MapLayer,
                                          image: &impl ImageCollection, texture: &egui::TextureHandle, canvas_rect: Rect) {
        let compatible_layer = self.edit_layer == layer || self.edit_layer == MapLayer::Screen;
        if compatible_layer && let MapSelection::LayerFragment(pos, frag) = &self.selection {
            let frag_x = pos.x as i32;
            let frag_y = pos.y as i32;
            for y in 0..frag.height {
                let tile_y = { let tile_y = frag_y + y as i32; if tile_y < 0 { continue; } tile_y as u32 };
                for x in 0..frag.width {
                    let tile_x = { let tile_x = frag_x + x as i32; if tile_x < 0 { continue; } tile_x as u32 };
                    let tile = frag.get_tile(x, y) as u32;
                    if tile == 0xff || tile >= image.num_items() { continue; }
                    let tile_rect = Self::get_tile_rect(tile_x, tile_y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile));
                    image.paint_at(ui, tile_rect);
                }
            }
            return;
        }

        if let MapSelection::FullFragment(pos, frag) = &self.selection && (layer != MapLayer::Background || ! frag.bg_data.is_empty()) {
            let frag_x = pos.x as i32;
            let frag_y = pos.y as i32;
            for y in 0..frag.height {
                let tile_y = { let tile_y = frag_y + y as i32; if tile_y < 0 { continue; } tile_y as u32 };
                for x in 0..frag.width {
                    let tile_x = { let tile_x = frag_x + x as i32; if tile_x < 0 { continue; } tile_x as u32 };
                    let tile = frag.get_layer_tile(x, y, layer) as u32;
                    if tile == 0xff || tile >= image.num_items() { continue; }
                    let tile_rect = Self::get_tile_rect(tile_x, tile_y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile));
                    image.paint_at(ui, tile_rect);
                }
            }
        }
    }

    fn handle_selection_mouse(&mut self, pointer_pos: Pos2, resp: &egui::Response, map_data: &mut MapData,
                              canvas_to_map_fg: &emath::RectTransform, canvas_to_map_bg: &emath::RectTransform) {
        let (mouse_pos, map_size) = match self.edit_layer {
            MapLayer::Background => (canvas_to_map_bg * pointer_pos, canvas_to_map_bg.to().size()),
            _ => (canvas_to_map_fg * pointer_pos, canvas_to_map_fg.to().size()),
        };
        if ! resp.dragged_by(egui::PointerButton::Primary) {
            if ! resp.dragged_by(egui::PointerButton::Secondary) && ! resp.dragged_by(egui::PointerButton::Middle) {
                self.drag_mouse_origin = mouse_pos;
                self.drag_frag_origin = mouse_pos;
            }
            return;
        }

        let orig_mouse_pos = mouse_pos;
        let mouse_pos = Rect::from_min_size(Pos2::ZERO, map_size).clamp(mouse_pos);
        if resp.drag_started() {
            self.drag_mouse_origin = mouse_pos;
            match self.selection {
                MapSelection::Rect(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() && sel_rect.contains(orig_mouse_pos) {
                        self.lift_selection(map_data);
                        self.drag_frag_origin = sel_rect.min;
                    } else {
                        self.selection = MapSelection::None;
                    }
                }
                MapSelection::LayerFragment(..) | MapSelection::FullFragment(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() {
                        if sel_rect.contains(orig_mouse_pos) {
                            self.drag_frag_origin = sel_rect.min;
                        } else {
                            self.drop_selection(map_data);
                        }
                    }
                }
                _ => {}
            }
        } else if ! resp.drag_stopped() {
            self.selection = if let Some((_, frag)) = self.selection.take_layer_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                MapSelection::LayerFragment(pos, frag)
            } else if let Some((_, frag)) = self.selection.take_full_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                MapSelection::FullFragment(pos, frag)
            } else {
                MapSelection::Rect(self.drag_mouse_origin.round(), mouse_pos.round())
            };
        }
    }

    fn set_fg_layer_tile(&self, layer: MapLayer, pos: Pos2, tile: u32, map_data: &mut MapData) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.width || y >= map_data.height { return; }
        match layer {
            MapLayer::Foreground => { map_data.fg_tiles[(map_data.width * y + x) as usize] = tile as u8; }
            MapLayer::Clip => { map_data.clip_tiles[(map_data.width * y + x) as usize] = tile as u8; }
            MapLayer::Effects => { map_data.fx_tiles[(map_data.width * y + x) as usize] = tile as u8; }
            _ => {}
        }
    }

    fn set_bg_layer_tile(&self, pos: Pos2, tile: u32, map_data: &mut MapData) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.bg_width || y >= map_data.bg_height { return; }
        map_data.bg_tiles[(map_data.bg_width * y + x) as usize] = tile as u8;
    }

    fn handle_mouse(&mut self, pointer_pos: Pos2, response: &egui::Response, map_data: &mut MapData,
                    canvas_to_map_fg: &emath::RectTransform, canvas_to_map_bg: &emath::RectTransform) {
        match self.tool {
            MapTool::Pencil => {
                if let Some(tile) = self.get_selected_tile_for_click(response) {
                    match self.edit_layer {
                        MapLayer::Foreground | MapLayer::Clip | MapLayer::Effects => {
                            if response.drag_started() { self.set_undo_target(map_data); }
                            self.set_fg_layer_tile(self.edit_layer, canvas_to_map_fg * pointer_pos, tile, map_data);
                        }
                        MapLayer::Background => {
                            if response.drag_started() { self.set_undo_target(map_data); }
                            self.set_bg_layer_tile(canvas_to_map_bg * pointer_pos, tile, map_data);
                        }
                        MapLayer::Screen => {
                            self.screen_display_pos = (canvas_to_map_fg * pointer_pos * TILE_SIZE - 0.5 * SCREEN_SIZE).to_vec2();
                        }
                    }
                }
            }

            MapTool::SelectLayer | MapTool::SelectFgLayers | MapTool::SelectAllLayers => {
                self.handle_selection_mouse(pointer_pos, response, map_data, canvas_to_map_fg, canvas_to_map_bg);
            }
        }
    }

    fn get_tile_rect(x: u32, y: u32, zoom: f32, canvas_pos: Pos2) -> Rect {
        let pos = Vec2 {
            x: x as f32 * TILE_SIZE * zoom,
            y: y as f32 * TILE_SIZE * zoom,
        };
        Rect {
            min: canvas_pos + pos,
            max: canvas_pos + pos + zoom * Vec2::splat(TILE_SIZE),
        }
    }

    pub fn undo(&mut self, map_data: &mut MapData) {
        if let Some(undo_target) = self.undo_target.take() {
            undo_target.to_map(map_data);
            self.selection = MapSelection::None;
        }
    }

    pub fn cut(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        self.set_undo_target(map_data);
        self.lift_selection(map_data);
        if let Some((_, frag)) = self.selection.take_layer_fragment() {
            wc.map_clipboard = MapClipboardData::MapLayerFragment(frag);
            return;
        }
        if let Some((_, frag)) = self.selection.take_full_fragment() {
            wc.map_clipboard = MapClipboardData::MapFullFragment(frag);
        }
    }

    pub fn copy(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        if self.tool == MapTool::SelectLayer && self.edit_layer == MapLayer::Screen { return; }

        match &self.selection {
            MapSelection::LayerFragment(_, frag) => {
                wc.map_clipboard = MapClipboardData::MapLayerFragment(frag.clone());
            }
            MapSelection::FullFragment(_, frag) => {
                wc.map_clipboard = MapClipboardData::MapFullFragment(frag.clone());
            }
            MapSelection::Rect(..) => {
                if let Some(sel_rect) = self.selection.get_rect() &&
                    sel_rect.is_positive() &&
                    let Some(map_rect) = MapRect::from_rect(sel_rect, map_data, self.edit_layer) {
                        match self.tool {
                            MapTool::SelectLayer => {
                                if let Some(frag) = MapLayerFragment::copy_map(map_data, self.edit_layer, map_rect) {
                                    wc.map_clipboard = MapClipboardData::MapLayerFragment(frag.clone());
                                }
                            }

                            MapTool::SelectAllLayers | MapTool::SelectFgLayers => {
                                let include_bg_layer = self.tool == MapTool::SelectAllLayers;
                                if let Some(frag) = MapFullFragment::copy_map(map_data, map_rect, include_bg_layer) {
                                    wc.map_clipboard = MapClipboardData::MapFullFragment(frag.clone());
                                }
                            }

                            _ => {}
                        }
                    }
            }
            _ => {}
        }
    }

    pub fn paste(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        match &wc.map_clipboard {
            MapClipboardData::MapLayerFragment(frag) => {
                self.tool = MapTool::SelectLayer;
                self.edit_layer = frag.layer;
                self.set_undo_target(map_data);
                self.drop_selection(map_data);
                self.selection = MapSelection::LayerFragment(Pos2::ZERO, frag.clone());
            }

            MapClipboardData::MapFullFragment(frag) => {
                let full_bg = map_data.bg_width == map_data.width || map_data.bg_height == map_data.height;
                if frag.bg_data.is_empty() && (full_bg || self.tool != MapTool::SelectAllLayers) {
                    MapTool::SelectFgLayers
                } else {
                    MapTool::SelectAllLayers
                };
                self.set_undo_target(map_data);
                self.drop_selection(map_data);
                self.selection = MapSelection::FullFragment(Pos2::ZERO, frag.clone());
            }

            _ => {}
        }
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &mut MapData) {
        let ctrl_z = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&ctrl_z)) {
            self.undo(map_data);
            return;
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(map_data);
        }

        match wc.keyboard_pressed.take() {
            Some(KeyboardPressed::CtrlC) => { self.copy(wc, map_data); }
            Some(KeyboardPressed::CtrlX) => { self.cut(wc, map_data); }
            Some(KeyboardPressed::CtrlV) => { self.paste(wc, map_data); }
            None => {}
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &mut MapData, tileset: &Tileset) {
        let min_size = (self.zoom * Vec2::splat(TILE_SIZE)).max(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let canvas_rect = Rect {
            min: response_rect.min.floor(),
            max: response_rect.max.floor(),
        };
        let zoomed_tile_size = self.zoom * TILE_SIZE;
        let map_size = Vec2 {
            x: map_data.width as f32 * zoomed_tile_size,
            y: map_data.height as f32 * zoomed_tile_size,
        };
        let map_bg_size = Vec2 {
            x: map_data.bg_width as f32 * zoomed_tile_size,
            y: map_data.bg_height as f32 * zoomed_tile_size,
        };
        let bg_rect = if map_size.x >= canvas_rect.width() && map_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect {
                min: canvas_rect.min,
                max: canvas_rect.min + map_size.min(canvas_rect.size()),
            }
        };
        let canvas_to_map_fg = emath::RectTransform::from_to(
            Rect { min: canvas_rect.min + self.scroll, max: canvas_rect.min + map_size + self.scroll },
            Rect { min: Pos2::ZERO, max: Pos2::new(map_data.width as f32, map_data.height as f32) }
        );
        let canvas_to_map_bg = emath::RectTransform::from_to(
            Rect { min: canvas_rect.min + self.scroll, max: canvas_rect.min + map_bg_size + self.scroll },
            Rect { min: Pos2::ZERO, max: Pos2::new(map_data.bg_width as f32, map_data.bg_height as f32) }
        );

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), map_size);

        // ensure we don't draw outside the map area
        ui.shrink_clip_rect(canvas_rect);

        if self.tool_changed || self.edit_layer_changed {
            self.tool_changed = false;
            self.edit_layer_changed = false;
            self.drop_selection(map_data);
        }

        // draw background green
        painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0xffu8, 0));

        // background
        if self.display.has_bits(MapDisplay::BACKGROUND) {
            let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
            for y in 0..map_data.bg_height {
                for x in 0..map_data.bg_width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                    if tile == 0xff || tile >= tileset.num_tiles { continue; }
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile));
                    if self.edit_layer == MapLayer::Foreground {
                        image.tint(Color32::BLACK).paint_at(ui, tile_rect);
                    } else {
                        image.paint_at(ui, tile_rect);
                    }
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Background, tileset, texture, canvas_rect);
        }

        // foreground
        if self.display.has_bits(MapDisplay::FOREGROUND) {
            let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                    if tile == 0xff || tile >= tileset.num_tiles { continue; }
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile));
                    if self.edit_layer == MapLayer::Background {
                        image.tint(Color32::BLACK).paint_at(ui, tile_rect);
                    } else {
                        image.paint_at(ui, tile_rect);
                    }
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Foreground, tileset, texture, canvas_rect);
        }

        // collision
        if self.display.has_bits(MapDisplay::CLIP) {
            let clip_tiles = STATIC_IMAGES.clip_tiles();
            let texture = clip_tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Clip, x, y);
                    if tile == 0xff || tile >= clip_tiles.num_items { continue; }
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(clip_tiles.get_item_uv(tile)).paint_at(ui, tile_rect);
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Clip, clip_tiles, texture, canvas_rect);
        }

        // effects
        if self.display.has_bits(MapDisplay::EFFECTS) {
            let fx_tiles = STATIC_IMAGES.fx_tiles();
            let texture = fx_tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Effects, x, y);
                    if tile == 0xff || tile >= fx_tiles.num_items { continue; }
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(fx_tiles.get_item_uv(tile)).paint_at(ui, tile_rect);
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Effects, fx_tiles, texture, canvas_rect);
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, self.custom_grid_color.unwrap_or(wc.settings.map_grid_color));
        if self.display.has_bits(MapDisplay::GRID) {
            for y in 0..map_data.height+1 {
                let cy = canvas_rect.min.y + y as f32 * zoomed_tile_size + self.scroll.y%zoomed_tile_size;
                painter.hline(bg_rect.x_range(), cy, stroke);
            }
            for x in 0..map_data.width+1 {
                let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + self.scroll.x % zoomed_tile_size;
                painter.vline(cx, bg_rect.y_range(), stroke);
            }
        }
        let border_rect = bg_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // screen size
        if self.display.has_bits(MapDisplay::SCREEN) {
            let stroke1 = egui::Stroke::new(3.0, Color32::PURPLE);
            let stroke2 = egui::Stroke::new(1.0, Color32::YELLOW);
            let pos = canvas_rect.min + self.zoom * self.screen_display_pos + self.scroll;
            let screen_rect = Rect {
                min: pos,
                max: pos + self.zoom * SCREEN_SIZE,
            };
            painter.rect_stroke(screen_rect, egui::CornerRadius::ZERO, stroke1, egui::StrokeKind::Middle);
            painter.rect_stroke(screen_rect, egui::CornerRadius::ZERO, stroke2, egui::StrokeKind::Middle);
        }

        // check zoom
        if let (true, Some(hover_pos)) = (
            response.contains_pointer(),
            ui.input(|i| i.pointer.hover_pos()),
        ) {
            let zoom_delta = ui.input(|i| i.zoom_delta());  // or use i.smooth_scroll_delta if CTRL key is should not be required?
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, map_data);
            }
            self.hover_pos = ((hover_pos - canvas_rect.min - self.scroll) / self.zoom / TILE_SIZE).max(Vec2::ZERO);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), map_size);
        }

        // check click
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            self.handle_mouse(pointer_pos, &response, map_data, &canvas_to_map_fg, &canvas_to_map_bg);
        }

        // draw selection rectangle
        if let Some(sel_rect) = self.selection.get_rect() && (sel_rect.width() > 0.0 || sel_rect.height() > 0.0) {
            let map_to_canvas = match self.edit_layer {
                MapLayer::Background => canvas_to_map_bg.inverse(),
                _ => canvas_to_map_fg.inverse(),
            };
            let sel_rect = Rect {
                min: map_to_canvas * sel_rect.min,
                max: map_to_canvas * sel_rect.max,
            };
            if sel_rect.is_positive() || response.dragged_by(egui::PointerButton::Primary) {
                super::paint_marching_ants(&painter, sel_rect, wc.settings);
                wc.request_marching_ants_repaint();
            }
        }
    }
}
