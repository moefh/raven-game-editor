use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::{MapData, Tileset};
use egui::{Vec2, Sense, Rect, Pos2, Color32, Image};
use egui::emath;

use super::{MapLayer, TILE_SIZE, SCREEN_SIZE, get_map_layer_tile};

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
    Select,
}

pub struct MapEditorState {
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
}

impl MapEditorState {
    pub fn new() -> Self {
        MapEditorState {
            zoom: 2.0,
            scroll: Vec2::ZERO,
            edit_layer: MapLayer::Background,
            tool: MapTool::Pencil,
            display: MapDisplay::new(MapDisplay::FOREGROUND | MapDisplay::BACKGROUND | MapDisplay::GRID),
            left_draw_tile: 0,
            right_draw_tile: 0xff,
            hover_pos: Vec2::ZERO,
            custom_grid_color: None,
            screen_display_pos: Vec2::splat(TILE_SIZE/2.0),
        }
    }

    pub fn set_edit_layer(&mut self, layer: MapLayer) {
        self.edit_layer = layer;
    }

    pub fn set_tool(&mut self, tool: MapTool) {
        self.tool = tool;
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

    fn get_selected_tile_for_click(&self, response: &egui::Response) -> Option<u32> {
        if response.dragged_by(egui::PointerButton::Primary) {
            Some(self.left_draw_tile)
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            Some(self.right_draw_tile)
        } else {
            None
        }
    }

    fn handle_mouse_select(&mut self, _pointer_pos: Pos2, _response: &egui::Response, _map_data: &mut MapData,
                           _canvas_to_map_fg: &emath::RectTransform, _canvas_to_map_bg: &emath::RectTransform) {
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
                            self.set_fg_layer_tile(self.edit_layer, canvas_to_map_fg * pointer_pos, tile, map_data);
                        }
                        MapLayer::Background => {
                            self.set_bg_layer_tile(canvas_to_map_bg * pointer_pos, tile, map_data);
                        }
                        MapLayer::Screen => {
                            self.screen_display_pos = (canvas_to_map_fg * pointer_pos * TILE_SIZE - 0.5 * SCREEN_SIZE).to_vec2();
                        }
                    }
                }
            }

            MapTool::Select => { self.handle_mouse_select(pointer_pos, response, map_data, canvas_to_map_fg, canvas_to_map_bg); }
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

pub fn map_editor(ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &mut MapData, tileset: &Tileset, state: &mut MapEditorState) {
    let image = ImageCollection::from_asset(tileset);
    let min_size = (state.zoom * Vec2::splat(TILE_SIZE)).max(ui.available_size());
    let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
    let response_rect = response.rect;

    let canvas_rect = Rect {
        min: response_rect.min.floor(),
        max: response_rect.max.floor(),
    };
    let zoomed_tile_size = state.zoom * TILE_SIZE;
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
        Rect { min: canvas_rect.min + state.scroll, max: canvas_rect.min + map_size + state.scroll },
        Rect { min: Pos2::ZERO, max: Pos2::new(map_data.width as f32, map_data.height as f32) }
    );
    let canvas_to_map_bg = emath::RectTransform::from_to(
        Rect { min: canvas_rect.min + state.scroll, max: canvas_rect.min + map_bg_size + state.scroll },
        Rect { min: Pos2::ZERO, max: Pos2::new(map_data.bg_width as f32, map_data.bg_height as f32) }
    );

    // limit scroll in case we've been resized
    state.clip_scroll(canvas_rect.size(), map_size);

    // ensure we don't draw outside the map area
    ui.shrink_clip_rect(canvas_rect);

    // draw background green
    painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0xffu8, 0));

    // background
    if state.display.has_bits(MapDisplay::BACKGROUND) {
        let texture = image.texture(wc.tex_man, wc.egui.ctx, tileset, TextureSlot::Opaque);
        for y in 0..map_data.bg_height {
            for x in 0..map_data.bg_width {
                let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                if tile == 0xff || tile >= image.num_items { continue; }
                let tile_rect = get_tile_rect(x, y, state.zoom, canvas_rect.min + state.scroll);
                let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile));
                if state.edit_layer == MapLayer::Foreground {
                    image.tint(Color32::BLACK).paint_at(ui, tile_rect);
                } else {
                    image.paint_at(ui, tile_rect);
                }
            }
        }
    }

    // foreground
    if state.display.has_bits(MapDisplay::FOREGROUND) {
        let texture = image.texture(wc.tex_man, wc.egui.ctx, tileset, TextureSlot::Transparent);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                if tile == 0xff || tile >= image.num_items { continue; }
                let tile_rect = get_tile_rect(x, y, state.zoom, canvas_rect.min + state.scroll);
                let image = Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile));
                if state.edit_layer == MapLayer::Background {
                    image.tint(Color32::BLACK).paint_at(ui, tile_rect);
                } else {
                    image.paint_at(ui, tile_rect);
                }
            }
        }
    }

    // collision
    if state.display.has_bits(MapDisplay::CLIP) {
        // TODO
    }

    // effects
    if state.display.has_bits(MapDisplay::EFFECTS) {
        // TODO
    }

    // grid and border
    let stroke = egui::Stroke::new(1.0, state.custom_grid_color.unwrap_or(wc.settings.map_grid_color));
    if state.display.has_bits(MapDisplay::GRID) {
        for y in 0..map_data.height+1 {
            let cy = canvas_rect.min.y + y as f32 * zoomed_tile_size + state.scroll.y%zoomed_tile_size;
            painter.hline(bg_rect.x_range(), cy, stroke);
        }
        for x in 0..map_data.width+1 {
            let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + state.scroll.x % zoomed_tile_size;
            painter.vline(cx, bg_rect.y_range(), stroke);
        }
    }
    let border_rect = bg_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
    painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

    // screen size
    if state.display.has_bits(MapDisplay::SCREEN) {
        let stroke1 = egui::Stroke::new(3.0, Color32::PURPLE);
        let stroke2 = egui::Stroke::new(1.0, Color32::YELLOW);
        let pos = canvas_rect.min + state.zoom * state.screen_display_pos + state.scroll;
        let screen_rect = Rect {
            min: pos,
            max: pos + state.zoom * SCREEN_SIZE,
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
            state.set_zoom(state.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, map_data);
        }
        state.hover_pos = ((hover_pos - canvas_rect.min - state.scroll) / state.zoom / TILE_SIZE).max(Vec2::ZERO);
    }

    // check pan
    if response.dragged_by(egui::PointerButton::Middle) {
        state.scroll += response.drag_delta();
        state.clip_scroll(canvas_rect.size(), map_size);
    }

    // check click
    if let Some(pointer_pos) = response.interact_pointer_pos() {
        state.handle_mouse(pointer_pos, &response, map_data, &canvas_to_map_fg, &canvas_to_map_bg);
    }
}
