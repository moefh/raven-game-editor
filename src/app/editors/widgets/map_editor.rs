use std::collections::VecDeque;

use egui::{
    emath,
    Vec2,
    Sense,
    Rect,
    Pos2,
    Color32,
    Image,
};

use crate::platform::current_time_as_millis;
use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::data_asset::{
    AssetList,
    MapData,
    Tileset,
    TileAnimation,
};
use crate::misc::STATIC_IMAGES;

use super::{
    TILE_SIZE,
    SCREEN_SIZE,
    get_map_layer_tile,
};
use super::super::{
    get_animated_tile,
    WindowContext,
    KeyboardPressed,
    MapClipboardData,
    MapUndoData,
    MapWholeFragment,
    MapLayerFragment,
    MapRect,
    MapLayer,
};

const FULL_UV: Rect = Rect { min: Pos2::ZERO, max: Pos2 { x: 1.0, y: 1.0 } };

pub enum MapSelection {
    None,
    Rect(Pos2, Pos2),
    LayerFragment(Pos2, MapLayerFragment),
    WholeFragment(Pos2, MapWholeFragment),
}

impl MapSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, MapSelection::LayerFragment(..) | MapSelection::WholeFragment(..))
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
            MapSelection::WholeFragment(_, _) => false,
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

    pub fn take_whole_fragment(&mut self) -> Option<(Pos2, MapWholeFragment)> {
        match self {
            MapSelection::WholeFragment(..) => {
                let mut ret = MapSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    MapSelection::WholeFragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen, but it's not a big deal
                }
            }
            _ => None
        }
    }

    pub fn get_rect(&self) -> Option<Rect> {
        match self {
            MapSelection::Rect(origin, end) => {
                Some(Rect::from_min_max(origin.min(*end), origin.max(*end)))
            }
            MapSelection::LayerFragment(pos, frag) => {
                Some(Rect::from_min_size(*pos, Vec2::new(frag.width as f32, frag.height as f32)))
            }
            MapSelection::WholeFragment(pos, frag) => {
                Some(Rect::from_min_size(*pos, Vec2::new(frag.width as f32, frag.height as f32)))
            }
            MapSelection::None => None,
        }
    }

    pub fn get_tile_planes<'a>(&'a mut self, planes: &mut Vec<&'a mut [u8]>) {
        match self {
            MapSelection::None => {}
            MapSelection::Rect(..) => {}
            MapSelection::LayerFragment(_, frag) => {
                match frag.layer {
                    MapLayer::Effects | MapLayer::Animation | MapLayer::Screen => {}
                    MapLayer::Foreground | MapLayer::Background | MapLayer::Parallax => {
                        planes.push(&mut frag.data)
                    }
                }
            }
            MapSelection::WholeFragment(_, frag) => {
                planes.push(&mut frag.fg_data);
                planes.push(&mut frag.bg_data);
                planes.push(&mut frag.para_data);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct MapDisplay {
    bits: u32,
}

impl MapDisplay {
    pub const FOREGROUND: u32    = 1 << 0;
    pub const BACKGROUND: u32    = 1 << 1;
    pub const EFFECTS: u32       = 1 << 2;
    pub const ANIMATION: u32     = 1 << 3;
    pub const PARALLAX: u32      = 1 << 4;
    pub const GRID: u32          = 1 << 5;
    pub const SCREEN: u32        = 1 << 6;
    pub const ANIMATE_TILES: u32 = 1 << 7;

    pub fn new(bits: u32) -> Self {
        MapDisplay {
            bits,
        }
    }

    pub fn toggle(&mut self, bits: u32) {
        self.bits ^= bits;
    }

    pub fn set(&mut self, bits: u32) {
        self.bits |= bits;
    }

    pub fn has_bits(&self, bits: u32) -> bool {
        (self.bits & bits) != 0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapTool {
    Pencil,
    SelectLayer,
    SelectFullLayers,
    SelectAllLayers,
}

pub struct MapEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub edit_layer: MapLayer,
    pub display: MapDisplay,
    pub tool: MapTool,
    pub left_draw_tile: u8,
    pub right_draw_tile: u8,
    pub left_draw_tile_changed: bool,
    pub right_draw_tile_changed: bool,
    pub hover_pos: Vec2,
    pub custom_grid_color: Option<Color32>,
    pub custom_bg_color: Option<Color32>,
    pub screen_display_pos: Pos2,
    pub selection: MapSelection,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
    tool_changed: bool,
    edit_layer_changed: bool,
    undo_targets: VecDeque<MapUndoData>,
    redo_targets: VecDeque<MapUndoData>,
    tool_mouse_down: bool,
}

impl MapEditorWidget {
    const MAX_UNDO_TARGETS: usize = 32;
    const LIGHT_LAYER_TINT: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 96);
    const HEAVY_LAYER_TINT: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 128);

    pub fn new() -> Self {
        let zoom = 0.75;
        MapEditorWidget {
            zoom,
            scroll: Vec2::ZERO,
            edit_layer: MapLayer::Screen,
            tool: MapTool::Pencil,
            display: MapDisplay::new(MapDisplay::FOREGROUND | MapDisplay::BACKGROUND | MapDisplay::GRID),
            left_draw_tile: 0,
            right_draw_tile: MapData::NO_TILE,
            left_draw_tile_changed: false,
            right_draw_tile_changed: false,
            hover_pos: Vec2::ZERO,
            custom_grid_color: None,
            custom_bg_color: None,
            screen_display_pos: Pos2::new(TILE_SIZE/zoom, TILE_SIZE/zoom),
            selection: MapSelection::None,
            drag_mouse_origin: Pos2::ZERO,
            drag_frag_origin: Pos2::ZERO,
            tool_changed: false,
            edit_layer_changed: false,
            undo_targets: VecDeque::new(),
            redo_targets: VecDeque::new(),
            tool_mouse_down: false,
        }
    }

    pub fn get_tile_planes(&mut self) -> Vec<&mut [u8]> {
        let mut ret = Vec::<&mut [u8]>::new();
        self.selection.get_tile_planes(&mut ret);
        for undo in self.undo_targets.iter_mut() {
            ret.push(&mut undo.fg_tiles);
            ret.push(&mut undo.bg_tiles);
            ret.push(&mut undo.para_tiles);
        }
        for redo in self.redo_targets.iter_mut() {
            ret.push(&mut redo.fg_tiles);
            ret.push(&mut redo.bg_tiles);
            ret.push(&mut redo.para_tiles);
        }
        ret
    }

    pub fn set_undo_target(&mut self, map_data: &MapData) {
        self.redo_targets.clear();
        if self.undo_targets.len() >= Self::MAX_UNDO_TARGETS {
            self.undo_targets.pop_front();
        }
        self.undo_targets.push_back(MapUndoData::from_map(map_data));
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

        let fill_tile = self.get_fill_tile_for_layer();
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

                    MapTool::SelectAllLayers | MapTool::SelectFullLayers => {
                        self.set_undo_target(map_data);
                        let include_para_layer = self.tool == MapTool::SelectAllLayers;
                        if let Some(frag) = MapWholeFragment::cut_map(map_data, map_rect, fill_tile, include_para_layer) {
                            MapSelection::WholeFragment(sel_rect.min, frag)
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
            MapSelection::WholeFragment(pos, frag) => {
                frag.paste_in_map(pos.x as i32, pos.y as i32, map_data);
            }
            _ => {}
        }
        self.selection = MapSelection::None;
    }

    fn get_selected_tile_for_click(&self, response: &egui::Response) -> Option<u8> {
        if response.dragged_by(egui::PointerButton::Primary) {
            Some(self.left_draw_tile)
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            Some(self.right_draw_tile)
        } else {
            None
        }
    }

    fn set_selected_tile_for_click(&mut self, tile: u8, response: &egui::Response) {
        if response.dragged_by(egui::PointerButton::Primary) {
            self.left_draw_tile = tile;
            self.left_draw_tile_changed = true;
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            self.right_draw_tile = tile;
            self.right_draw_tile_changed = true;
        }
    }

    fn get_fill_tile_for_layer(&self) -> u8 {
        match self.edit_layer {
            MapLayer::Foreground | MapLayer::Effects => MapData::NO_TILE,
            MapLayer::Background | MapLayer::Parallax => self.right_draw_tile,
            _ => MapData::NO_TILE,
        }
    }

    fn paint_floating_selection_for_layer(
        &self,
        ui: &mut egui::Ui,
        layer: MapLayer,
        wc: &mut WindowContext,
        image: &impl ImageCollection,
        slot: TextureSlot,
        canvas_rect: Rect
    ) {
        let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);

        let compatible_layer = self.edit_layer == layer || self.edit_layer == MapLayer::Screen;
        if compatible_layer && let MapSelection::LayerFragment(pos, frag) = &self.selection {
            let frag_x = pos.x as i32;
            let frag_y = pos.y as i32;
            for y in 0..frag.height {
                let tile_y = { let tile_y = frag_y + y as i32; if tile_y < 0 { continue; } tile_y as u32 };
                for x in 0..frag.width {
                    let tile_x = { let tile_x = frag_x + x as i32; if tile_x < 0 { continue; } tile_x as u32 };
                    let tile = frag.get_tile(x, y);
                    if tile == MapData::NO_TILE || tile as u32 >= image.num_items() { continue; }
                    let tile_rect = Self::get_tile_rect(tile_x, tile_y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile as u32));
                    image.paint_at(ui, tile_rect);
                }
            }
            return;
        }

        if let MapSelection::WholeFragment(pos, frag) = &self.selection && (layer != MapLayer::Background || ! frag.para_data.is_empty()) {
            let frag_x = pos.x as i32;
            let frag_y = pos.y as i32;
            for y in 0..frag.height {
                let tile_y = { let tile_y = frag_y + y as i32; if tile_y < 0 { continue; } tile_y as u32 };
                for x in 0..frag.width {
                    let tile_x = { let tile_x = frag_x + x as i32; if tile_x < 0 { continue; } tile_x as u32 };
                    let tile = frag.get_layer_tile(x, y, layer);
                    if tile == MapData::NO_TILE || tile as u32 >= image.num_items() { continue; }
                    let tile_rect = Self::get_tile_rect(tile_x, tile_y, self.zoom, canvas_rect.min + self.scroll);
                    let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile as u32));
                    image.paint_at(ui, tile_rect);
                }
            }
        }
    }

    fn handle_selection_mouse(
        &mut self,
        pointer_pos: Pos2,
        resp: &egui::Response,
        map_data: &mut MapData,
        canvas_to_map_full: &emath::RectTransform,
        canvas_to_map_para: &emath::RectTransform
    ) {
        let (mouse_pos, map_size) = match self.edit_layer {
            MapLayer::Parallax => (canvas_to_map_para * pointer_pos, canvas_to_map_para.to().size()),
            _ => (canvas_to_map_full * pointer_pos, canvas_to_map_full.to().size()),
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
                MapSelection::LayerFragment(..) | MapSelection::WholeFragment(..) => {
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
            } else if let Some((_, frag)) = self.selection.take_whole_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                MapSelection::WholeFragment(pos, frag)
            } else {
                MapSelection::Rect(self.drag_mouse_origin.round(), mouse_pos.round())
            };
        }
    }

    fn set_full_layer_tile(&self, layer: MapLayer, pos: Pos2, tile: u8, map_data: &mut MapData) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.width || y >= map_data.height { return; }
        match layer {
            MapLayer::Foreground => { map_data.fg_tiles[(map_data.width * y + x) as usize] = tile; }
            MapLayer::Background => { map_data.bg_tiles[(map_data.width * y + x) as usize] = tile; }
            MapLayer::Effects => {
                map_data.fx_tiles[(map_data.width * y + x) as usize] &= 0xf0;
                map_data.fx_tiles[(map_data.width * y + x) as usize] |= tile & 0x0f;
            }
            MapLayer::Animation => {
                map_data.fx_tiles[(map_data.width * y + x) as usize] &= 0x0f;
                map_data.fx_tiles[(map_data.width * y + x) as usize] |= (tile & 0x0f) << 4;
            }
            _ => {}
        }
    }

    fn get_full_layer_tile(&self, layer: MapLayer, pos: Pos2, map_data: &mut MapData) -> Option<u8> {
        if pos.x < 0.0 || pos.y < 0.0 { return None; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.width || y >= map_data.height { return None; }
        match layer {
            MapLayer::Foreground => { Some(map_data.fg_tiles[(map_data.width * y + x) as usize]) }
            MapLayer::Background => { Some(map_data.bg_tiles[(map_data.width * y + x) as usize]) }
            MapLayer::Effects => {
                let fx = map_data.fx_tiles[(map_data.width * y + x) as usize] & 0x0f;
                if fx != 0x0f { Some(fx) } else { Some(0xff) }
            }
            MapLayer::Animation => {
                let anim = map_data.fx_tiles[(map_data.width * y + x) as usize] >> 4;
                if anim != 0x0f { Some(anim) } else { Some(0xff) }
            }
            _ => { None }
        }
    }

    fn set_para_layer_tile(&self, pos: Pos2, tile: u8, map_data: &mut MapData) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.para_width || y >= map_data.para_height { return; }
        map_data.para_tiles[(map_data.para_width * y + x) as usize] = tile;
    }

    fn get_para_layer_tile(&self, pos: Pos2, map_data: &mut MapData) -> Option<u8> {
        if pos.x < 0.0 || pos.y < 0.0 { return None; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= map_data.para_width || y >= map_data.para_height {
            None
        } else {
            Some(map_data.para_tiles[(map_data.para_width * y + x) as usize])
        }
    }

    fn handle_mouse(
        &mut self,
        pointer_pos: Pos2,
        response: &egui::Response,
        map_data: &mut MapData,
        canvas_to_map_full: &emath::RectTransform,
        canvas_to_map_para: &emath::RectTransform
    ) {
        if matches!(self.edit_layer, MapLayer::Screen) {
            if ! response.dragged_by(egui::PointerButton::Primary) { return; }
            let mouse_pos = canvas_to_map_full * pointer_pos * TILE_SIZE;
            if response.drag_started() {
                let display_rect = egui::Rect::from_min_size(self.screen_display_pos, SCREEN_SIZE);
                if ! display_rect.contains(mouse_pos) {
                    self.screen_display_pos = mouse_pos - 0.5 * SCREEN_SIZE;
                }
                self.drag_mouse_origin = mouse_pos;
                self.drag_frag_origin = self.screen_display_pos;
            } else {
                self.screen_display_pos = self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin);
            }
            return;
        }

        let keys_pressed = response.ctx.input(|i| i.modifiers);

        match self.tool {
            MapTool::Pencil => {
                if keys_pressed.command {
                    let pick_tile = match self.edit_layer {
                        MapLayer::Foreground | MapLayer::Background | MapLayer::Effects | MapLayer::Animation => {
                            self.get_full_layer_tile(self.edit_layer, canvas_to_map_full * pointer_pos, map_data)
                        }
                        MapLayer::Parallax => {
                            self.get_para_layer_tile(canvas_to_map_para * pointer_pos, map_data)
                        }
                        MapLayer::Screen => {
                            None
                        }
                    };
                    if let Some(tile) = pick_tile {
                        self.set_selected_tile_for_click(tile, response);
                    }
                } else if let Some(tile) = self.get_selected_tile_for_click(response) {
                    match self.edit_layer {
                        MapLayer::Foreground | MapLayer::Background | MapLayer::Effects | MapLayer::Animation => {
                            if response.drag_started() { self.set_undo_target(map_data); }
                            self.set_full_layer_tile(self.edit_layer, canvas_to_map_full * pointer_pos, tile, map_data);
                        }
                        MapLayer::Parallax => {
                            if response.drag_started() { self.set_undo_target(map_data); }
                            self.set_para_layer_tile(canvas_to_map_para * pointer_pos, tile, map_data);
                        }
                        MapLayer::Screen => {}
                    }
                }
            }

            MapTool::SelectLayer | MapTool::SelectFullLayers | MapTool::SelectAllLayers => {
                self.handle_selection_mouse(pointer_pos, response, map_data, canvas_to_map_full, canvas_to_map_para);
            }
        }
    }

    fn get_tile_rect(x: u32, y: u32, zoom: f32, canvas_pos: Pos2) -> Rect {
        let pos = TILE_SIZE * zoom * Vec2::new(x as f32, y as f32);
        Rect::from_min_size(canvas_pos + pos, zoom * Vec2::splat(TILE_SIZE))
    }

    pub fn can_undo(&self) -> bool {
        ! self.undo_targets.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        ! self.redo_targets.is_empty()
    }

    pub fn undo(&mut self, map_data: &mut MapData) {
        if let Some(undo_target) = self.undo_targets.pop_back() {
            self.redo_targets.push_back(MapUndoData::from_map(map_data));
            undo_target.to_map(map_data);
            self.selection = MapSelection::None;
        }
    }

    pub fn redo(&mut self, map_data: &mut MapData) {
        if let Some(redo_target) = self.redo_targets.pop_back() {
            self.undo_targets.push_back(MapUndoData::from_map(map_data));
            redo_target.to_map(map_data);
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
        if let Some((_, frag)) = self.selection.take_whole_fragment() {
            wc.map_clipboard = MapClipboardData::MapWholeFragment(frag);
        }
    }

    pub fn copy(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        if self.tool == MapTool::SelectLayer && self.edit_layer == MapLayer::Screen { return; }

        match &self.selection {
            MapSelection::LayerFragment(_, frag) => {
                wc.map_clipboard = MapClipboardData::MapLayerFragment(frag.clone());
            }
            MapSelection::WholeFragment(_, frag) => {
                wc.map_clipboard = MapClipboardData::MapWholeFragment(frag.clone());
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

                            MapTool::SelectAllLayers | MapTool::SelectFullLayers => {
                                let include_para_layer = self.tool == MapTool::SelectAllLayers;
                                if let Some(frag) = MapWholeFragment::copy_map(map_data, map_rect, include_para_layer) {
                                    wc.map_clipboard = MapClipboardData::MapWholeFragment(frag.clone());
                                }
                            }

                            _ => {}
                        }
                    }
            }
            _ => {}
        }
    }

    fn get_paste_position(&self) -> Pos2 {
        Pos2 {
            x: ((-self.scroll.x / self.zoom + TILE_SIZE - 1.0) / TILE_SIZE).floor(),
            y: ((-self.scroll.y / self.zoom + TILE_SIZE - 1.0) / TILE_SIZE).floor(),
        }
    }

    pub fn paste(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        match &wc.map_clipboard {
            MapClipboardData::MapLayerFragment(frag) => {
                self.tool = MapTool::SelectLayer;
                self.edit_layer = frag.layer;
                self.set_undo_target(map_data);
                self.drop_selection(map_data);
                self.selection = MapSelection::LayerFragment(self.get_paste_position(), frag.clone());
            }

            MapClipboardData::MapWholeFragment(frag) => {
                let whole_frag = map_data.para_width == map_data.width || map_data.para_height == map_data.height;
                self.tool = if frag.para_data.is_empty() && (whole_frag || self.tool != MapTool::SelectAllLayers) {
                    MapTool::SelectFullLayers
                } else {
                    MapTool::SelectAllLayers
                };
                if self.edit_layer == MapLayer::Screen {
                    self.edit_layer = MapLayer::Foreground;
                }
                self.set_undo_target(map_data);
                self.drop_selection(map_data);
                self.selection = MapSelection::WholeFragment(self.get_paste_position(), frag.clone());
            }

            _ => {}
        }
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &mut MapData) {
        let cmd_shift_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND|egui::Modifiers::SHIFT, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_shift_z)) {
            self.redo(map_data);
            return;
        }

        let cmd_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_z)) {
            self.undo(map_data);
            return;
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(map_data);
        }

        match wc.keyboard_pressed.take() {
            Some(KeyboardPressed::CommandC) => { self.copy(wc, map_data); }
            Some(KeyboardPressed::CommandX) => { self.cut(wc, map_data); }
            Some(KeyboardPressed::CommandV) => { self.paste(wc, map_data); }
            None => {}
        }
    }

    fn calc_para_scroll(&self, map_area_rect: Rect, map_data: &MapData) -> Vec2 {
        let zoomed_tile_size = self.zoom * TILE_SIZE;
        let win_w = map_area_rect.width();
        let win_h = map_area_rect.height();
        let map_w = map_data.width as f32 * zoomed_tile_size;
        let map_h = map_data.height as f32 * zoomed_tile_size;
        let para_w = map_data.para_width as f32 * zoomed_tile_size;
        let para_h = map_data.para_height as f32 * zoomed_tile_size;
        let scroll_x = if map_w > win_w {
            self.scroll.x * (1.0 - (map_w - para_w) / (map_w - win_w))
        } else {
            0.0
        };
        let scroll_y = if map_h > win_h {
            self.scroll.y * (1.0 - (map_h - para_h) / (map_h - win_h))
        } else {
            0.0
        };
        Vec2::new(scroll_x, scroll_y)
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        map_data: &mut MapData,
        tilesets: &AssetList<Tileset>,
        tile_anims: &AssetList<TileAnimation>
    ) {
        let min_size = (self.zoom * Vec2::splat(TILE_SIZE)).max(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let canvas_rect = response_rect.expand2(Vec2::splat(-1.0));
        let zoomed_tile_size = self.zoom * TILE_SIZE;
        let map_size = Vec2 {
            x: map_data.width as f32 * zoomed_tile_size,
            y: map_data.height as f32 * zoomed_tile_size,
        };
        let map_para_size = Vec2 {
            x: map_data.para_width as f32 * zoomed_tile_size,
            y: map_data.para_height as f32 * zoomed_tile_size,
        };
        let map_area_rect = if map_size.x >= canvas_rect.width() && map_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect::from_min_size(canvas_rect.min, map_size.min(canvas_rect.size()))
        };

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), map_size);

        let canvas_to_map_full = emath::RectTransform::from_to(
            Rect::from_min_size(canvas_rect.min + self.scroll, map_size),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(map_data.width as f32, map_data.height as f32))
        );
        let para_scroll = self.calc_para_scroll(map_area_rect, map_data);
        let canvas_to_map_para = emath::RectTransform::from_to(
            Rect::from_min_size(canvas_rect.min + para_scroll, map_para_size),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(map_data.para_width as f32, map_data.para_height as f32))
        );

        // ensure we don't draw outside the map area
        ui.shrink_clip_rect(canvas_rect);

        if self.tool_changed || self.edit_layer_changed {
            self.tool_changed = false;
            self.edit_layer_changed = false;
            self.drop_selection(map_data);
        }

        // draw background green
        let bg_color = self.custom_bg_color.unwrap_or(wc.settings.map_bg_color);
        painter.rect_filled(map_area_rect, egui::CornerRadius::ZERO, bg_color);
        let mut has_animated_tiles = false;
        let animation_step = ((current_time_as_millis() / wc.settings.map_animation_ms_per_frame as u64) % 12) as u32;

        let (tile_anim, anim_tileset) = map_data
            .tile_anim_id
            .and_then(|tile_anim_id| tile_anims.get(&tile_anim_id))
            .map(|tile_anim| (Some(tile_anim), tilesets.get(&tile_anim.anim_tileset_id)))
            .unwrap_or((None, None));
        if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
            // parallax
            if self.display.has_bits(MapDisplay::PARALLAX) && map_data.para_width != 0 && map_data.para_height != 0 {
                for y in 0..map_data.para_height {
                    for x in 0..map_data.para_width {
                        let tile = get_map_layer_tile(map_data, MapLayer::Parallax, x, y);
                        if tile == MapData::NO_TILE { continue; }
                        let (uv, texture) = if tile as u32 >= tileset.num_tiles {
                            (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        } else {
                            (tileset.get_item_uv(tile as u32), tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque))
                        };
                        let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + para_scroll);
                        let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv);
                        if self.edit_layer == MapLayer::Foreground || self.edit_layer == MapLayer::Background {
                            image.tint(Self::LIGHT_LAYER_TINT).paint_at(ui, tile_rect);
                        } else {
                            image.paint_at(ui, tile_rect);
                        }
                    }
                }

                self.paint_floating_selection_for_layer(ui, MapLayer::Parallax, wc, tileset, TextureSlot::Opaque, canvas_rect);
            }

            // background
            if self.display.has_bits(MapDisplay::BACKGROUND) {
                for y in 0..map_data.height {
                    for x in 0..map_data.width {
                        let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                        if tile == MapData::NO_TILE { continue; }
                        let (tile, use_tileset) = if self.display.has_bits(MapDisplay::ANIMATE_TILES) &&
                            let Some(new_tile) = get_animated_tile(
                                tile,
                                MapLayer::Background,
                                get_map_layer_tile(map_data, MapLayer::Animation, x, y),
                                tile_anim,
                                animation_step
                            ) {
                                has_animated_tiles = true;
                                if let Some(anim_tileset) = anim_tileset {
                                    (new_tile, anim_tileset)
                                } else {
                                    (new_tile, tileset)
                                }
                            } else {
                                (tile, tileset)
                            };
                        let (uv, texture) = if tile as u32 >= use_tileset.num_tiles {
                            (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        } else {
                            (use_tileset.get_item_uv(tile as u32), use_tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque))
                        };
                        let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                        let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv);
                        let image = match self.edit_layer {
                            MapLayer::Foreground => { image.tint(Self::LIGHT_LAYER_TINT) }
                            MapLayer::Parallax => { image.tint(Self::HEAVY_LAYER_TINT) }
                            _ => { image }
                        };
                        image.paint_at(ui, tile_rect);
                    }
                }

                self.paint_floating_selection_for_layer(ui, MapLayer::Background, wc, tileset, TextureSlot::Opaque, canvas_rect);
            }

            // foreground
            if self.display.has_bits(MapDisplay::FOREGROUND) {
                for y in 0..map_data.height {
                    for x in 0..map_data.width {
                        let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                        if tile == MapData::NO_TILE { continue; }
                        let (tile, use_tileset) = if self.display.has_bits(MapDisplay::ANIMATE_TILES) &&
                            let Some(new_tile) = get_animated_tile(
                                tile,
                                MapLayer::Foreground,
                                get_map_layer_tile(map_data, MapLayer::Animation, x, y),
                                tile_anim,
                                animation_step
                            ) {
                                has_animated_tiles = true;
                                if let Some(anim_tileset) = anim_tileset {
                                    (new_tile, anim_tileset)
                                } else {
                                    (new_tile, tileset)
                                }
                            } else {
                                (tile, tileset)
                            };
                        let (uv, texture) = if tile as u32 >= use_tileset.num_tiles {
                            (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        } else {
                            (use_tileset.get_item_uv(tile as u32), use_tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        };
                        let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                        let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv);
                        let image = match self.edit_layer {
                            MapLayer::Background => { image.tint(Self::LIGHT_LAYER_TINT) }
                            MapLayer::Parallax => { image.tint(Self::HEAVY_LAYER_TINT) }
                            _ => { image }
                        };
                        image.paint_at(ui, tile_rect);
                    }
                }

                self.paint_floating_selection_for_layer(ui, MapLayer::Foreground, wc, tileset, TextureSlot::Opaque, canvas_rect);
            }
        }

        // effects
        if self.display.has_bits(MapDisplay::EFFECTS) {
            let fx_tiles = STATIC_IMAGES.fx_tiles();
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Effects, x, y);
                    if tile == MapData::NO_TILE { continue; }
                    let (uv, texture) = if tile as u32 >= fx_tiles.num_items {
                        (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                    } else {
                        (fx_tiles.get_item_uv(tile as u32), fx_tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                    };
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv).paint_at(ui, tile_rect);
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Effects, wc, fx_tiles, TextureSlot::Transparent, canvas_rect);
        }

        // animations
        if self.display.has_bits(MapDisplay::ANIMATION) {
            let anim_tiles = STATIC_IMAGES.anim_tiles();
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    let tile = get_map_layer_tile(map_data, MapLayer::Animation, x, y);
                    if tile == MapData::NO_TILE { continue; }
                    let (uv, texture) = if tile as u32 >= anim_tiles.num_items {
                        (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                    } else {
                        (anim_tiles.get_item_uv(tile as u32), anim_tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                    };
                    let tile_rect = Self::get_tile_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv).paint_at(ui, tile_rect);
                }
            }

            self.paint_floating_selection_for_layer(ui, MapLayer::Animation, wc, anim_tiles, TextureSlot::Transparent, canvas_rect);
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, self.custom_grid_color.unwrap_or(wc.settings.map_grid_color));
        if self.display.has_bits(MapDisplay::GRID) {
            for y in 0..map_data.height+1 {
                let cy = canvas_rect.min.y + y as f32 * zoomed_tile_size + self.scroll.y%zoomed_tile_size;
                painter.hline(map_area_rect.x_range(), cy, stroke);
            }
            for x in 0..map_data.width+1 {
                let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + self.scroll.x % zoomed_tile_size;
                painter.vline(cx, map_area_rect.y_range(), stroke);
            }
        }
        let border_rect = map_area_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // screen size
        if self.display.has_bits(MapDisplay::SCREEN) {
            let stroke1 = egui::Stroke::new(3.0, Color32::PURPLE);
            let stroke2 = egui::Stroke::new(1.0, Color32::YELLOW);
            let pos = canvas_rect.min + self.zoom * self.screen_display_pos.to_vec2() + self.scroll;
            let screen_rect = Rect::from_min_size(pos, self.zoom * SCREEN_SIZE);
            painter.rect_stroke(screen_rect, egui::CornerRadius::ZERO, stroke1, egui::StrokeKind::Middle);
            painter.rect_stroke(screen_rect, egui::CornerRadius::ZERO, stroke2, egui::StrokeKind::Middle);
        }

        if has_animated_tiles {
            wc.request_map_animation_repaint();
        }

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // set cursor
        if response.contains_pointer() && response.hovered() {
            if keys_pressed.alt {
                response.ctx.set_cursor_icon(egui::CursorIcon::AllScroll);
            } else if keys_pressed.command {
                response.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
            } else if matches!(self.edit_layer, MapLayer::Screen) {
                response.ctx.set_cursor_icon(egui::CursorIcon::AllScroll);
            } else {
                match self.tool {
                    MapTool::Pencil => {}
                    MapTool::SelectLayer | MapTool::SelectFullLayers | MapTool::SelectAllLayers => {
                        response.ctx.set_cursor_icon(egui::CursorIcon::Crosshair)
                    }
                }
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, map_data);
            }
            self.hover_pos = ((hover_pos - canvas_rect.min - self.scroll) / self.zoom / TILE_SIZE).max(Vec2::ZERO);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged() && keys_pressed.alt) {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), map_size);
        }

        // check click
        if response.drag_stopped() {
            self.tool_mouse_down = false;
        }
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! keys_pressed.alt {
            if response.drag_started() {
                self.tool_mouse_down = true;
            }
            if self.tool_mouse_down {
                self.handle_mouse(pointer_pos, &response, map_data, &canvas_to_map_full, &canvas_to_map_para);
            }
        }

        // draw selection rectangle
        if let Some(sel_rect) = self.selection.get_rect() && (sel_rect.width() > 0.0 || sel_rect.height() > 0.0) {
            let map_to_canvas = match self.edit_layer {
                MapLayer::Parallax => canvas_to_map_para.inverse(),
                _ => canvas_to_map_full.inverse(),
            };
            let sel_rect = map_to_canvas.transform_rect(sel_rect);
            if sel_rect.is_positive() || response.dragged_by(egui::PointerButton::Primary) {
                super::paint_marching_ants(&painter, sel_rect, wc.settings);
                wc.request_marching_ants_repaint();
            }
        }
    }
}
