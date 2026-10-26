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
    MapData,
    MapTileChange,
    Tileset,
};
use crate::misc::STATIC_IMAGES;

use super::{
    TILE_SIZE,
    ImagePickerSelection,
};
use super::super::{
    WindowContext,
    MapTileChangeUndoData,
    MapRect,
    MapTileChangeFragment,
    DrawMapInfo,
};

const FULL_UV: Rect = Rect { min: Pos2::ZERO, max: Pos2 { x: 1.0, y: 1.0 } };

#[derive(Clone, Copy)]
pub struct MapTileChangeDisplay {
    bits: u32,
}

#[allow(unused)]
impl MapTileChangeDisplay {
    pub const GRID: u32        = 1 << 0;
    pub const TRANSPARENT: u32 = 1 << 1;

    pub fn new(bits: u32) -> Self {
        MapTileChangeDisplay {
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

pub enum MapTileChangeSelection {
    None,
    Rect(Pos2, Pos2),
    LayerFragment(Pos2, MapTileChangeFragment),
}

impl MapTileChangeSelection {
    pub fn is_floating(&self) -> bool {
        matches!(self, MapTileChangeSelection::LayerFragment(..))
    }

    pub fn take_layer_fragment(&mut self) -> Option<(Pos2, MapTileChangeFragment)> {
        match self {
            MapTileChangeSelection::LayerFragment(..) => {
                let mut ret = MapTileChangeSelection::None;
                std::mem::swap(self, &mut ret);
                match ret {
                    MapTileChangeSelection::LayerFragment(pos, frag) => Some((pos, frag)),
                    _ => None,  // this shouldn't happen, but it's not a big deal
                }
            }
            _ => None
        }
    }

    pub fn get_rect(&self) -> Option<Rect> {
        match self {
            MapTileChangeSelection::Rect(origin, end) => {
                Some(Rect::from_min_max(origin.min(*end), origin.max(*end)))
            }
            MapTileChangeSelection::LayerFragment(pos, frag) => {
                Some(Rect::from_min_size(*pos, Vec2::new(frag.width as f32, frag.height as f32)))
            }
            MapTileChangeSelection::None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapTileChangeTool {
    Pencil,
    Select,
}

pub struct MapTileChangeEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub display: MapTileChangeDisplay,
    pub tool: MapTileChangeTool,
    pub left_draw_tile: ImagePickerSelection,
    pub right_draw_tile: ImagePickerSelection,
    pub left_draw_tile_changed: bool,
    pub right_draw_tile_changed: bool,
    pub hover_pos: Vec2,
    pub hover_tile: Option<ImagePickerSelection>,
    pub custom_grid_color: Option<Color32>,
    pub custom_bg_color: Option<Color32>,
    pub selection: MapTileChangeSelection,
    drag_mouse_origin: Pos2,
    drag_frag_origin: Pos2,
    tool_changed: bool,
    edit_layer_changed: bool,
    undo_targets: VecDeque<MapTileChangeUndoData>,
    redo_targets: VecDeque<MapTileChangeUndoData>,
    tool_mouse_down: bool,
    last_map_area_size: Vec2,
}

impl MapTileChangeEditorWidget {
    const DEFAULT_DISPLAY: u32 = MapTileChangeDisplay::GRID | MapTileChangeDisplay::TRANSPARENT;
    const MAX_UNDO_TARGETS: usize = 32;

    pub fn new() -> Self {
        let zoom = 3.0;
        MapTileChangeEditorWidget {
            zoom,
            scroll: Vec2::ZERO,
            display: MapTileChangeDisplay::new(Self::DEFAULT_DISPLAY),
            tool: MapTileChangeTool::Pencil,
            left_draw_tile: ImagePickerSelection::Some(0),
            right_draw_tile: ImagePickerSelection::None,
            left_draw_tile_changed: false,
            right_draw_tile_changed: false,
            hover_pos: Vec2::ZERO,
            hover_tile: None,
            custom_grid_color: None,
            custom_bg_color: None,
            selection: MapTileChangeSelection::None,
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

    pub fn clear_undo(&mut self) {
        self.undo_targets.clear();
        self.redo_targets.clear();
    }

    fn selection_to_tile_change(sel: ImagePickerSelection) -> Option<u8> {
        match sel {
            ImagePickerSelection::Some(tile) => { Some((tile & 0xff) as u8) }
            ImagePickerSelection::None => { Some(0xff) }
            ImagePickerSelection::Empty(_) => { None }
        }
    }

    pub fn set_undo_target(&mut self, change: &MapTileChange) {
        self.redo_targets.clear();
        if self.undo_targets.len() >= Self::MAX_UNDO_TARGETS {
            self.undo_targets.pop_front();
        }
        self.undo_targets.push_back(MapTileChangeUndoData::from_tile_change(change));
    }

    pub fn set_tool(&mut self, tool: MapTileChangeTool) {
        self.tool = tool;
        self.tool_changed = true;
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, change: &MapTileChange) {
        let zoom = zoom.max(0.25);
        let change_size = TILE_SIZE * zoom * Vec2::new(
            change.width as f32,
            change.height as f32,
        );
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, change_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, change_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - change_size).min(Vec2::ZERO);
    }

    pub fn lift_selection(&mut self, change: &mut MapTileChange) {
        if self.selection.is_floating() { return; } // already floating

        let fill_tile = Self::selection_to_tile_change(self.right_draw_tile);
        if let Some(sel_rect) = self.selection.get_rect() &&
            sel_rect.is_positive() &&
            let Some(change_rect) = MapRect::from_tile_change(sel_rect, change) {
                self.selection = match self.tool {
                    MapTileChangeTool::Select => {
                        self.set_undo_target(change);
                        if let Some(frag) = MapTileChangeFragment::cut_change(change, change_rect, fill_tile) {
                            MapTileChangeSelection::LayerFragment(sel_rect.min, frag)
                        } else {
                            MapTileChangeSelection::None
                        }
                    }

                    _ => MapTileChangeSelection::None
                }
            }
    }

    pub fn delete_selection(&mut self, change: &mut MapTileChange) {
        self.lift_selection(change);
        self.selection = MapTileChangeSelection::None;
    }

    pub fn drop_selection(&mut self, change: &mut MapTileChange) {
        if let MapTileChangeSelection::LayerFragment(pos, frag) = &self.selection {
            frag.paste_in_change(pos.x as i32, pos.y as i32, change);
        }
        self.selection = MapTileChangeSelection::None;
    }

    fn get_selected_tile_for_click(&self, response: &egui::Response) -> Option<ImagePickerSelection> {
        if response.dragged_by(egui::PointerButton::Primary) {
            Some(self.left_draw_tile)
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            Some(self.right_draw_tile)
        } else {
            None
        }
    }

    fn set_selected_tile_for_click(&mut self, tile: ImagePickerSelection, response: &egui::Response) {
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

        if let MapTileChangeSelection::LayerFragment(pos, frag) = &self.selection {
            let frag_x = pos.x as i32;
            let frag_y = pos.y as i32;
            for y in 0..frag.height {
                let tile_y = { let tile_y = frag_y + y as i32; if tile_y < 0 { continue; } tile_y as u32 };
                for x in 0..frag.width {
                    let tile_x = { let tile_x = frag_x + x as i32; if tile_x < 0 { continue; } tile_x as u32 };
                    if let Some(tile) = frag.get_tile(x, y) {
                        if tile == MapData::NO_TILE || tile as u32 >= image.num_items() { continue; }
                        let tile_rect = Self::get_tile_rect(tile_x, tile_y, self.zoom, canvas_rect.min + self.scroll);
                        let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile as u32));
                        image.paint_at(ui, tile_rect);
                    }
                }
            }
        }
    }

    fn handle_selection_mouse(
        &mut self,
        pointer_pos: Pos2,
        resp: &egui::Response,
        change: &mut MapTileChange,
        canvas_to_change: &emath::RectTransform
    ) {
        let mouse_pos = canvas_to_change * pointer_pos;
        let change_size = canvas_to_change.to().size();
        if ! resp.dragged_by(egui::PointerButton::Primary) {
            if ! resp.dragged_by(egui::PointerButton::Secondary) && ! resp.dragged_by(egui::PointerButton::Middle) {
                self.drag_mouse_origin = mouse_pos;
                self.drag_frag_origin = mouse_pos;
            }
            return;
        }

        let orig_mouse_pos = mouse_pos;
        let mouse_pos = Rect::from_min_size(Pos2::ZERO, change_size).clamp(mouse_pos);
        if resp.drag_started() {
            self.drag_mouse_origin = mouse_pos;
            match self.selection {
                MapTileChangeSelection::Rect(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() && sel_rect.contains(orig_mouse_pos) {
                        self.lift_selection(change);
                        self.drag_frag_origin = sel_rect.min;
                    } else {
                        self.selection = MapTileChangeSelection::None;
                    }
                }
                MapTileChangeSelection::LayerFragment(..) => {
                    if let Some(sel_rect) = self.selection.get_rect() {
                        if sel_rect.contains(orig_mouse_pos) {
                            self.drag_frag_origin = sel_rect.min;
                        } else {
                            self.drop_selection(change);
                        }
                    }
                }
                _ => {}
            }
        } else if ! resp.drag_stopped() {
            self.selection = if let Some((_, frag)) = self.selection.take_layer_fragment() {
                let pos = (self.drag_frag_origin + (mouse_pos - self.drag_mouse_origin)).round();
                MapTileChangeSelection::LayerFragment(pos, frag)
            } else {
                MapTileChangeSelection::Rect(self.drag_mouse_origin.round(), mouse_pos.round())
            };
        }
    }

    fn set_tile(&self, pos: Pos2, tile: ImagePickerSelection, change: &mut MapTileChange) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= change.width || y >= change.height { return; }
        change.tiles[(change.width * y + x) as usize] = Self::selection_to_tile_change(tile);
    }

    fn get_tile(&self, pos: Pos2, change: &mut MapTileChange) -> Option<ImagePickerSelection> {
        if pos.x < 0.0 || pos.y < 0.0 { return None; }
        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x >= change.width || y >= change.height {
            None
        } else if let Some(tile) = change.tiles[(change.width * y + x) as usize] {
            if tile != 0xff {
                Some(ImagePickerSelection::Some(tile as u32))
            } else {
                Some(ImagePickerSelection::None)
            }
        } else {
            Some(ImagePickerSelection::Empty(0))
        }
    }

    fn handle_mouse(
        &mut self,
        pointer_pos: Pos2,
        response: &egui::Response,
        change: &mut MapTileChange,
        canvas_to_change: &emath::RectTransform
    ) {
        let keys_pressed = response.ctx.input(|i| i.modifiers);

        match self.tool {
            MapTileChangeTool::Pencil => {
                if keys_pressed.command {
                    if let Some(sel) = self.get_tile(canvas_to_change * pointer_pos, change) {
                        self.set_selected_tile_for_click(sel, response);
                    }
                } else if let Some(sel) = self.get_selected_tile_for_click(response) {
                    if response.drag_started() { self.set_undo_target(change); }
                    self.set_tile(canvas_to_change * pointer_pos, sel, change);
                }
            }

            MapTileChangeTool::Select => {
                self.handle_selection_mouse(pointer_pos, response, change, canvas_to_change);
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

    pub fn undo(&mut self, change: &mut MapTileChange) {
        if let Some(undo_target) = self.undo_targets.pop_back() {
            self.redo_targets.push_back(MapTileChangeUndoData::from_tile_change(change));
            undo_target.to_tile_change(change);
            self.selection = MapTileChangeSelection::None;
        }
    }

    pub fn redo(&mut self, change: &mut MapTileChange) {
        if let Some(redo_target) = self.redo_targets.pop_back() {
            self.undo_targets.push_back(MapTileChangeUndoData::from_tile_change(change));
            redo_target.to_tile_change(change);
            self.selection = MapTileChangeSelection::None;
        }
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, change: &mut MapTileChange) {
        let cmd_shift_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND|egui::Modifiers::SHIFT, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_shift_z)) {
            self.redo(change);
            return;
        }

        let cmd_z = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
        if ui.input_mut(|i| i.consume_shortcut(&cmd_z)) {
            self.undo(change);
            return;
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) {
            self.delete_selection(change);
        }

        // tools
        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::A))) {
            self.set_tool(MapTileChangeTool::Pencil);
        }
        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::S))) {
            self.set_tool(MapTileChangeTool::Select);
        }
    }

    fn draw_tiles(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        change: &MapTileChange,
        tileset: &Tileset,
        info: &DrawMapInfo
    ) {
        let slot = if self.display.has_bits(MapTileChangeDisplay::TRANSPARENT) {
            TextureSlot::Transparent
        } else {
            TextureSlot::Opaque
        };
        for y in 0..change.height {
            for x in 0..change.width {
                let tile = change.tiles[(y * change.width + x) as usize];
                if tile.is_some_and(|t| t == MapData::NO_TILE) { continue; }
                let (uv, texture) = if let Some(tile) = tile {
                    if tile as u32 >= tileset.num_tiles {
                        (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                    } else {
                        (tileset.get_item_uv(tile as u32), tileset.texture(wc.tex_man, wc.egui.ctx, slot))
                    }
                } else {
                    (FULL_UV, STATIC_IMAGES.no_change_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque))
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
        change: &mut MapTileChange,
        tileset: &Tileset
    ) {
        let min_size = (self.zoom * Vec2::splat(TILE_SIZE)).max(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let canvas_rect = response_rect.expand2(Vec2::splat(-1.0));
        let zoomed_tile_size = self.zoom * TILE_SIZE;
        let change_size = Vec2 {
            x: change.width as f32 * zoomed_tile_size,
            y: change.height as f32 * zoomed_tile_size,
        };
        let change_area_rect = if change_size.x >= canvas_rect.width() && change_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect::from_min_size(canvas_rect.min, change_size.min(canvas_rect.size()))
        };

        self.last_map_area_size = change_area_rect.size();

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), change_size);

        let canvas_to_change = emath::RectTransform::from_to(
            Rect::from_min_size(canvas_rect.min + self.scroll, change_size),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(change.width as f32, change.height as f32))
        );

        // ensure we don't draw outside the map area
        ui.shrink_clip_rect(canvas_rect);

        if self.tool_changed || self.edit_layer_changed {
            self.tool_changed = false;
            self.edit_layer_changed = false;
            self.drop_selection(change);
        }

        // draw background green
        let bg_color = self.custom_bg_color.unwrap_or(
            if self.display.has_bits(MapTileChangeDisplay::TRANSPARENT) {
                wc.settings.image_bg_color
            } else {
                egui::Color32::from_rgb(0, 255, 0)
            }
        );
        painter.rect_filled(change_area_rect, egui::CornerRadius::ZERO, bg_color);

        let draw_info = DrawMapInfo {
            zoom: self.zoom,
            pos: canvas_rect.min + self.scroll,
            screen_rect: change_area_rect,
            animation_step: None,
        };

        if change.width != 0 && change.height != 0 {
            self.draw_tiles(ui, wc, change, tileset, &draw_info);
            self.paint_floating_selection(ui, wc, tileset, TextureSlot::Opaque, canvas_rect);
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, self.custom_grid_color.unwrap_or(wc.settings.map_grid_color));
        if self.display.has_bits(MapTileChangeDisplay::GRID) {
            for y in 0..change.height+1 {
                let cy = canvas_rect.min.y + y as f32 * zoomed_tile_size + self.scroll.y%zoomed_tile_size;
                painter.hline(change_area_rect.x_range(), cy, stroke);
            }
            for x in 0..change.width+1 {
                let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + self.scroll.x % zoomed_tile_size;
                painter.vline(cx, change_area_rect.y_range(), stroke);
            }
        }
        let border_rect = change_area_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
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
                    MapTileChangeTool::Pencil => {}
                    MapTileChangeTool::Select => {
                        response.ctx.set_cursor_icon(egui::CursorIcon::Crosshair)
                    }
                }
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, change);
            }
            self.hover_pos = ((hover_pos - canvas_rect.min - self.scroll) / self.zoom / TILE_SIZE).max(Vec2::ZERO);
            self.hover_tile = self.get_tile(self.hover_pos.to_pos2(), change);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged() && keys_pressed.alt) {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), change_size);
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
                self.handle_mouse(pointer_pos, &response, change, &canvas_to_change);
            }
        }

        // draw selection rectangle
        if let Some(sel_rect) = self.selection.get_rect() && (sel_rect.width() > 0.0 || sel_rect.height() > 0.0) {
            let sel_rect = canvas_to_change.inverse().transform_rect(sel_rect);
            if sel_rect.is_positive() || response.dragged_by(egui::PointerButton::Primary) {
                super::paint_marching_ants(&painter, sel_rect, wc.settings);
                wc.request_marching_ants_repaint();
            }
        }
    }
}
