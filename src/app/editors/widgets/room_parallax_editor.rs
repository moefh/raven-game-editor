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

use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::data_store::{
    DataAssetList,
    MapData,
    Room,
    RoomParallax,
    Tileset,
};
use crate::misc::STATIC_IMAGES;

use super::{
    TILE_SIZE,
};
use super::super::{
    WindowContext,
    KeyboardPressed,
    MapClipboardData,
    RoomParallaxUndoData,
    MapRect,
    MapLayerFragment,
    DrawMapInfo,
};

const FULL_UV: Rect = Rect { min: Pos2::ZERO, max: Pos2 { x: 1.0, y: 1.0 } };

#[derive(Clone, Copy)]
pub struct RoomParallaxDisplay {
    bits: u32,
}

#[allow(unused)]
impl RoomParallaxDisplay {
    pub const GRID: u32            = 1 << 4;

    pub fn new(bits: u32) -> Self {
        RoomParallaxDisplay {
            bits,
        }
    }

    pub fn toggle(&mut self, bits: u32) {
        self.bits ^= bits;
    }

    pub fn has_bits(&self, bits: u32) -> bool {
        (self.bits & bits) != 0
    }
}

pub enum RoomParallaxSelection {
    None,
    Rect(Pos2, Pos2),
    LayerFragment(Pos2, MapLayerFragment),
}

impl RoomParallaxSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, RoomParallaxSelection::LayerFragment(..))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            RoomParallaxSelection::None => true,
            RoomParallaxSelection::Rect(origin, end) => {
                let width = end.x - origin.x;
                let height = end.y - origin.y;
                width.abs() == 0.0 && height.abs() == 0.0
            }
            RoomParallaxSelection::LayerFragment(_, _) => false,
        }
    }

    pub fn take_layer_fragment(&mut self) -> Option<(Pos2, MapLayerFragment)> {
        match self {
            RoomParallaxSelection::LayerFragment(..) => {
                let mut ret = RoomParallaxSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    RoomParallaxSelection::LayerFragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen, but it's not a big deal
                }
            }
            _ => None
        }
    }

    pub fn get_rect(&self) -> Option<Rect> {
        match self {
            RoomParallaxSelection::Rect(origin, end) => {
                Some(Rect::from_min_max(origin.min(*end), origin.max(*end)))
            }
            RoomParallaxSelection::LayerFragment(pos, frag) => {
                Some(Rect::from_min_size(*pos, Vec2::new(frag.width as f32, frag.height as f32)))
            }
            RoomParallaxSelection::None => None,
        }
    }

    pub fn get_tile_planes<'a>(&'a mut self, planes: &mut Vec<&'a mut [u8]>) {
        if let RoomParallaxSelection::LayerFragment(_, frag) = self {
            planes.push(&mut frag.data)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RoomParallaxTool {
    Pencil,
    Select,
}

pub struct RoomParallaxEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub display: RoomParallaxDisplay,
    pub tool: RoomParallaxTool,
    pub left_draw_tile: u8,
    pub right_draw_tile: u8,
    pub left_draw_tile_changed: bool,
    pub right_draw_tile_changed: bool,
    pub hover_pos: Vec2,
    pub hover_tile: Option<u8>,
    pub custom_grid_color: Option<Color32>,
    pub custom_bg_color: Option<Color32>,
    pub selection: RoomParallaxSelection,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
    tool_changed: bool,
    edit_layer_changed: bool,
    undo_targets: VecDeque<RoomParallaxUndoData>,
    redo_targets: VecDeque<RoomParallaxUndoData>,
    tool_mouse_down: bool,
    last_map_area_size: Vec2,
}

impl RoomParallaxEditorWidget {
    const DEFAULT_DISPLAY: u32 = RoomParallaxDisplay::GRID;
    const MAX_UNDO_TARGETS: usize = 32;

    pub fn new() -> Self {
        let zoom = 0.75;
        RoomParallaxEditorWidget {
            zoom,
            scroll: Vec2::ZERO,
            display: RoomParallaxDisplay::new(Self::DEFAULT_DISPLAY),
            tool: RoomParallaxTool::Pencil,
            left_draw_tile: 0,
            right_draw_tile: MapData::NO_TILE,
            left_draw_tile_changed: false,
            right_draw_tile_changed: false,
            hover_pos: Vec2::ZERO,
            hover_tile: None,
            custom_grid_color: None,
            custom_bg_color: None,
            selection: RoomParallaxSelection::None,
            drag_mouse_origin: Pos2::ZERO,
            drag_frag_origin: Pos2::ZERO,
            tool_changed: false,
            edit_layer_changed: false,
            undo_targets: VecDeque::new(),
            redo_targets: VecDeque::new(),
            tool_mouse_down: false,
            last_map_area_size: Vec2::ZERO,
        }
    }

    pub fn get_tile_planes(&mut self) -> Vec<&mut [u8]> {
        let mut ret = Vec::<&mut [u8]>::new();
        self.selection.get_tile_planes(&mut ret);
        for undo in self.undo_targets.iter_mut() {
            ret.push(&mut undo.tiles);
        }
        for redo in self.redo_targets.iter_mut() {
            ret.push(&mut redo.tiles);
        }
        ret
    }

    pub fn set_undo_target(&mut self, room: &Room) {
        self.redo_targets.clear();
        if self.undo_targets.len() >= Self::MAX_UNDO_TARGETS {
            self.undo_targets.pop_front();
        }
        self.undo_targets.push_back(RoomParallaxUndoData::from_parallax(&room.parallax));
    }

    pub fn set_tool(&mut self, tool: RoomParallaxTool) {
        self.tool = tool;
        self.tool_changed = true;
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, room: &Room) {
        let zoom = zoom.max(0.25);
        let room_size = TILE_SIZE * zoom * Vec2::new(
            room.parallax.width as f32,
            room.parallax.height as f32,
        );
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, room_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, room_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - room_size).min(Vec2::ZERO);
    }

    pub fn lift_selection(&mut self, room: &mut Room) {
        if self.selection.is_floating() { return; } // already floating

        let fill_tile = self.right_draw_tile;
        if let Some(sel_rect) = self.selection.get_rect() &&
            sel_rect.is_positive() &&
            let Some(para_rect) = MapRect::from_parallax(sel_rect, &room.parallax) {
                self.selection = match self.tool {
                    RoomParallaxTool::Select => {
                        self.set_undo_target(room);
                        if let Some(frag) = MapLayerFragment::cut_room_parallax(&mut room.parallax, para_rect, fill_tile) {
                            RoomParallaxSelection::LayerFragment(sel_rect.min, frag)
                        } else {
                            RoomParallaxSelection::None
                        }
                    }

                    _ => RoomParallaxSelection::None
                }
            }
    }

    pub fn delete_selection(&mut self, room: &mut Room) {
        self.lift_selection(room);
        self.selection = RoomParallaxSelection::None;
    }

    pub fn drop_selection(&mut self, room: &mut Room) {
        if let RoomParallaxSelection::LayerFragment(pos, frag) = &self.selection {
            frag.paste_in_room_parallax(pos.x as i32, pos.y as i32, &mut room.parallax);
        }
        self.selection = RoomParallaxSelection::None;
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

    fn paint_floating_selection(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        image: &impl ImageCollection,
        slot: TextureSlot,
        canvas_rect: Rect
    ) {
        let texture = image.texture(wc.tex_man, wc.egui.ctx, slot);

        if let RoomParallaxSelection::LayerFragment(pos, frag) = &self.selection {
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
        }
    }

    fn handle_selection_mouse(
        &mut self,
        pointer_pos: Pos2,
        resp: &egui::Response,
        room: &mut Room,
        canvas_to_room: &emath::RectTransform
    ) {
        let mouse_pos = canvas_to_room * pointer_pos;
        let room_size = canvas_to_room.to().size();
        if ! resp.dragged_by(egui::PointerButton::Primary) {
            if ! resp.dragged_by(egui::PointerButton::Secondary) && ! resp.dragged_by(egui::PointerButton::Middle) {
                self.drag_mouse_origin = mouse_pos;
                self.drag_frag_origin = mouse_pos;
            }
            return;
        }

        let orig_mouse_pos = mouse_pos;
        let mouse_pos = Rect::from_min_size(Pos2::ZERO, room_size).clamp(mouse_pos);
        if resp.drag_started() {
            self.drag_mouse_origin = mouse_pos;
            match self.selection {
                RoomParallaxSelection::Rect(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() && sel_rect.contains(orig_mouse_pos) {
                        self.lift_selection(room);
                        self.drag_frag_origin = sel_rect.min;
                    } else {
                        self.selection = RoomParallaxSelection::None;
                    }
                }
                RoomParallaxSelection::LayerFragment(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() {
                        if sel_rect.contains(orig_mouse_pos) {
                            self.drag_frag_origin = sel_rect.min;
                        } else {
                            self.drop_selection(room);
                        }
                    }
                }
                _ => {}
            }
        } else if ! resp.drag_stopped() {
            self.selection = if let Some((_, frag)) = self.selection.take_layer_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                RoomParallaxSelection::LayerFragment(pos, frag)
            } else {
                RoomParallaxSelection::Rect(self.drag_mouse_origin.round(), mouse_pos.round())
            };
        }
    }

    fn set_tile(&self, pos: Pos2, tile: u8, room: &mut Room) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= room.parallax.width || y >= room.parallax.height { return; }
        room.parallax.tiles[(room.parallax.width * y + x) as usize] = tile;
    }

    fn get_tile(&self, pos: Pos2, room: &mut Room) -> Option<u8> {
        if pos.x < 0.0 || pos.y < 0.0 { return None; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= room.parallax.width || y >= room.parallax.height {
            None
        } else {
            Some(room.parallax.tiles[(room.parallax.width * y + x) as usize])
        }
    }

    fn handle_mouse(
        &mut self,
        pointer_pos: Pos2,
        response: &egui::Response,
        room: &mut Room,
        canvas_to_room: &emath::RectTransform
    ) {
        let keys_pressed = response.ctx.input(|i| i.modifiers);

        match self.tool {
            RoomParallaxTool::Pencil => {
                if keys_pressed.command {
                    if let Some(pick_tile) = self.get_tile(canvas_to_room * pointer_pos, room) {
                        self.set_selected_tile_for_click(pick_tile, response);
                    }
                } else if let Some(tile) = self.get_selected_tile_for_click(response) {
                    if response.drag_started() { self.set_undo_target(room); }
                    self.set_tile(canvas_to_room * pointer_pos, tile, room);
                }
            }

            RoomParallaxTool::Select => {
                self.handle_selection_mouse(pointer_pos, response, room, canvas_to_room);
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

    pub fn undo(&mut self, room: &mut Room) {
        if let Some(undo_target) = self.undo_targets.pop_back() {
            self.redo_targets.push_back(RoomParallaxUndoData::from_parallax(&room.parallax));
            undo_target.to_parallax(&mut room.parallax);
            self.selection = RoomParallaxSelection::None;
        }
    }

    pub fn redo(&mut self, room: &mut Room) {
        if let Some(redo_target) = self.redo_targets.pop_back() {
            self.undo_targets.push_back(RoomParallaxUndoData::from_parallax(&room.parallax));
            redo_target.to_parallax(&mut room.parallax);
            self.selection = RoomParallaxSelection::None;
        }
    }

    pub fn cut(&mut self, wc: &mut WindowContext, room: &mut Room) {
        self.set_undo_target(room);
        self.lift_selection(room);
        if let Some((_, frag)) = self.selection.take_layer_fragment() {
            wc.map_clipboard = MapClipboardData::MapLayerFragment(frag);
        }
    }

    pub fn copy(&mut self, wc: &mut WindowContext, room: &mut Room) {
        match &self.selection {
            RoomParallaxSelection::LayerFragment(_, frag) => {
                wc.map_clipboard = MapClipboardData::MapLayerFragment(frag.clone());
            }
            RoomParallaxSelection::Rect(..) => {
                if let Some(sel_rect) = self.selection.get_rect() &&
                    sel_rect.is_positive() &&
                    let Some(room_rect) = MapRect::from_parallax(sel_rect, &room.parallax) &&
                    self.tool == RoomParallaxTool::Select &&
                    let Some(frag) = MapLayerFragment::copy_room_parallax(&room.parallax, room_rect) {
                        wc.map_clipboard = MapClipboardData::MapLayerFragment(frag.clone());
                    }
            }
            _ => {}
        }
    }

    fn get_paste_position(&self, frag_size: Vec2) -> Pos2 {
        let top_left = Pos2 {
            x: (-self.scroll.x / self.zoom / TILE_SIZE).ceil(),
            y: (-self.scroll.y / self.zoom / TILE_SIZE).ceil(),
        };
        let pos = (0.5 * (self.last_map_area_size / self.zoom / TILE_SIZE - frag_size)).floor().max(Vec2::ZERO);
        top_left + pos
    }

    pub fn paste(&mut self, wc: &mut WindowContext, room: &mut Room) {
        if let MapClipboardData::MapLayerFragment(frag) = &wc.map_clipboard {
            self.tool = RoomParallaxTool::Select;
            self.set_undo_target(room);
            self.drop_selection(room);
            self.selection = RoomParallaxSelection::LayerFragment(self.get_paste_position(frag.size()), frag.clone());
        }
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, room: &mut Room) {
        let cmd_shift_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND|egui::Modifiers::SHIFT, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_shift_z)) {
            self.redo(room);
            return;
        }

        let cmd_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_z)) {
            self.undo(room);
            return;
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(room);
        }

        match wc.keyboard_pressed.take() {
            Some(KeyboardPressed::CommandC) => { self.copy(wc, room); }
            Some(KeyboardPressed::CommandX) => { self.cut(wc, room); }
            Some(KeyboardPressed::CommandV) => { self.paste(wc, room); }
            None => {}
        }
    }

    fn draw_tiles(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        para: &RoomParallax,
        tileset: &Tileset,
        info: &DrawMapInfo
    ) {
        for y in 0..para.height {
            for x in 0..para.width {
                let tile = para.tiles[(y * para.width + x) as usize];
                if tile == MapData::NO_TILE { continue; }
                let (uv, texture) = if tile as u32 >= tileset.num_tiles {
                    (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                } else {
                    (tileset.get_item_uv(tile as u32), tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque))
                };
                let tile_rect = egui::Rect::from_min_max(
                    info.pos + info.zoom * TILE_SIZE * egui::Vec2::new(x as f32, y as f32),
                    info.pos + info.zoom * TILE_SIZE * egui::Vec2::new((x+1) as f32, (y+1) as f32)
                );
                if tile_rect.intersects(info.screen_rect) {
                    let image = egui::Image::from_texture((texture.id(), egui::Vec2::splat(TILE_SIZE))).uv(uv);
                    image.paint_at(ui, tile_rect);
                }
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &mut Room,
        tilesets: &DataAssetList<Tileset>
    ) {
        if room.parallax.tileset_id.is_none() || room.parallax.width == 0 || room.parallax.height == 0 {
            ui.label("Room has no parallax");
            return;
        }

        let min_size = (self.zoom * Vec2::splat(TILE_SIZE)).max(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let canvas_rect = response_rect.expand2(Vec2::splat(-1.0));
        let zoomed_tile_size = self.zoom * TILE_SIZE;
        let room_size = Vec2 {
            x: room.parallax.width as f32 * zoomed_tile_size,
            y: room.parallax.height as f32 * zoomed_tile_size,
        };
        let room_area_rect = if room_size.x >= canvas_rect.width() && room_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect::from_min_size(canvas_rect.min, room_size.min(canvas_rect.size()))
        };

        self.last_map_area_size = room_area_rect.size();

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), room_size);

        let canvas_to_room = emath::RectTransform::from_to(
            Rect::from_min_size(canvas_rect.min + self.scroll, room_size),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(room.parallax.width as f32, room.parallax.height as f32))
        );

        // ensure we don't draw outside the map area
        ui.shrink_clip_rect(canvas_rect);

        if self.tool_changed || self.edit_layer_changed {
            self.tool_changed = false;
            self.edit_layer_changed = false;
            self.drop_selection(room);
        }

        // draw background green
        let bg_color = self.custom_bg_color.unwrap_or(wc.settings.map_bg_color);
        painter.rect_filled(room_area_rect, egui::CornerRadius::ZERO, bg_color);

        let draw_info = DrawMapInfo {
            zoom: self.zoom,
            pos: canvas_rect.min + self.scroll,
            screen_rect: room_area_rect,
            animation_step: None,
        };

        if let Some(tileset) = room.parallax.tileset_id.and_then(|tileset_id| tilesets.get(&tileset_id)) &&
            room.parallax.width != 0 &&
            room.parallax.height != 0 {
                self.draw_tiles(ui, wc, &room.parallax, tileset, &draw_info);
                self.paint_floating_selection(ui, wc, tileset, TextureSlot::Opaque, canvas_rect);
            }

        // grid and border
        let stroke = egui::Stroke::new(1.0, self.custom_grid_color.unwrap_or(wc.settings.map_grid_color));
        if self.display.has_bits(RoomParallaxDisplay::GRID) {
            for y in 0..room.parallax.height+1 {
                let cy = canvas_rect.min.y + y as f32 * zoomed_tile_size + self.scroll.y%zoomed_tile_size;
                painter.hline(room_area_rect.x_range(), cy, stroke);
            }
            for x in 0..room.parallax.width+1 {
                let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + self.scroll.x % zoomed_tile_size;
                painter.vline(cx, room_area_rect.y_range(), stroke);
            }
        }
        let border_rect = room_area_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // set cursor
        if response.contains_pointer() && response.hovered() {
            if keys_pressed.alt {
                response.ctx.set_cursor_icon(egui::CursorIcon::AllScroll);
            } else if keys_pressed.command {
                response.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
            } else {
                match self.tool {
                    RoomParallaxTool::Pencil => {}
                    RoomParallaxTool::Select => {
                        response.ctx.set_cursor_icon(egui::CursorIcon::Crosshair)
                    }
                }
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, room);
            }
            self.hover_pos = ((hover_pos - canvas_rect.min - self.scroll) / self.zoom / TILE_SIZE).max(Vec2::ZERO);
            self.hover_tile = self.get_tile(self.hover_pos.to_pos2(), room);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged() && keys_pressed.alt) {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), room_size);
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
                self.handle_mouse(pointer_pos, &response, room, &canvas_to_room);
            }
        }

        // draw selection rectangle
        if let Some(sel_rect) = self.selection.get_rect() && (sel_rect.width() > 0.0 || sel_rect.height() > 0.0) {
            let sel_rect = canvas_to_room.inverse().transform_rect(sel_rect);
            if sel_rect.is_positive() || response.dragged_by(egui::PointerButton::Primary) {
                super::paint_marching_ants(&painter, sel_rect, wc.settings);
                wc.request_marching_ants_repaint();
            }
        }
    }
}
