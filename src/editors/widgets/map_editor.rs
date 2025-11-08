use crate::app::ImageCollection;
use crate::data_asset::{MapData, Tileset};
use egui::{Vec2, Sense, Rect, Pos2, Color32, Image};
use egui::emath::GuiRounding;

const TILE_SIZE: Vec2 = Vec2::splat(Tileset::TILE_SIZE as f32);

const LAYER_MAP_FG: u32 = 0;
//const LAYER_MAP_CL: u32 = 1;
//const LAYER_MAP_FX: u32 = 2;
const LAYER_MAP_BG: u32 = 3;
const LAYER_GRID: u32 = 4;

const LAYER_FLAG_MAP_FG: u32 = 1 << LAYER_MAP_FG;
//const LAYER_FLAG_MAP_CL: u32 = 1 << LAYER_MAP_CL;
//const LAYER_FLAG_MAP_FX: u32 = 1 << LAYER_MAP_FX;
const LAYER_FLAG_MAP_BG: u32 = 1 << LAYER_MAP_BG;
const LAYER_FLAG_GRID: u32 = 1 << LAYER_GRID;

pub struct MapEditorState {
    pub zoom: f32,
    pub scroll: Vec2,
    pub edit_layer: u32,
    pub display_layers: u32,
    pub left_draw_tile: u32,
    pub right_draw_tile: u32,
}

impl MapEditorState {
    pub fn new() -> Self {
        MapEditorState {
            zoom: 2.0,
            scroll: Vec2::ZERO,
            edit_layer: LAYER_MAP_BG,
            display_layers: LAYER_FLAG_MAP_FG | LAYER_FLAG_MAP_BG | LAYER_FLAG_GRID,
            left_draw_tile: 0,
            right_draw_tile: 0xff,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, map_data: &MapData) {
        let zoom = zoom.max(0.25);
        let map_size = Vec2 {
            x: (map_data.width * Tileset::TILE_SIZE) as f32 * zoom,
            y: (map_data.height * Tileset::TILE_SIZE) as f32 * zoom,
        };
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, map_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, map_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - map_size).min(Vec2::ZERO);
    }
}

fn get_layer_tile(map_data: &MapData, layer: u32, x: u32, y: u32) -> u32 {
    if layer == LAYER_MAP_BG && (x >= map_data.bg_width || y >= map_data.bg_height) { return 0xff; }
    if x >= map_data.width || y >= map_data.height { return 0xff; }

    let base = layer * map_data.height * map_data.width;
    if layer == LAYER_MAP_BG {
        map_data.tiles[(base + map_data.bg_width * y + x) as usize] as u32
    } else {
        map_data.tiles[(base + map_data.width * y + x) as usize] as u32
    }
}

fn set_layer_tile(map_data: &mut MapData, x: i32, y: i32, layer: u32, tile: u32) {
    if x < 0 || y < 0 { return; }
    let x = x as u32;
    let y = y as u32;
    if layer == LAYER_MAP_BG && (x >= map_data.bg_width || y >= map_data.bg_height) { return; }
    if x >= map_data.width || y >= map_data.height { return; }

    let base = layer * map_data.height * map_data.width;
    if layer == LAYER_MAP_BG {
        map_data.tiles[(base + map_data.bg_width * y + x) as usize] = tile as u8;
    } else {
        map_data.tiles[(base + map_data.width * y + x) as usize] = tile as u8;
    }
}

fn get_tile_rect(x: u32, y: u32, zoom: f32, canvas_pos: Pos2) -> Rect {
    let pos = Vec2 {
        x: (x * Tileset::TILE_SIZE) as f32 * zoom,
        y: (y * Tileset::TILE_SIZE) as f32 * zoom,
    };
    Rect {
        min: canvas_pos + pos,
        max: canvas_pos + pos + zoom * TILE_SIZE,
    }
}

pub fn map_editor(ui: &mut egui::Ui, map_data: &mut MapData, texture: &egui::TextureHandle,
                  image: &ImageCollection, state: &mut MapEditorState) {
    //let tile_size = ;
    let min_size = (state.zoom * TILE_SIZE).max(ui.available_size());
    let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
    let response_rect = response.rect;

    let canvas_rect = Rect {
        min: response_rect.min.floor(),
        max: response_rect.max.floor(),
    }.round_to_pixels(ui.pixels_per_point());
    let zoomed_tile_size = Tileset::TILE_SIZE as f32 * state.zoom;
    let map_size = Vec2 {
        x: map_data.width as f32 * zoomed_tile_size,
        y: map_data.height as f32 * zoomed_tile_size,
    };
    let bg_rect = if map_size.x >= canvas_rect.width() && map_size.y >= canvas_rect.height() {
        canvas_rect
    } else {
        Rect {
            min: canvas_rect.min,
            max: canvas_rect.min + map_size.min(canvas_rect.size()),
        }.round_to_pixels(ui.pixels_per_point())
    };

    // limit scroll in case we've been resized
    state.clip_scroll(canvas_rect.size(), map_size);

    // ensure we don't draw outside the map area
    ui.shrink_clip_rect(canvas_rect);

    // draw background green
    painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0xffu8, 0));

    // background
    if (state.display_layers & LAYER_FLAG_MAP_BG) != 0 {
        for y in 0..map_data.bg_height {
            for x in 0..map_data.bg_width {
                let tile = get_layer_tile(map_data, LAYER_MAP_BG, x, y);
                if tile == 0xff || tile >= image.num_items { continue; }
                let tile_rect = get_tile_rect(x, y, state.zoom, canvas_rect.min + state.scroll).round_to_pixels(ui.pixels_per_point());
                Image::from_texture((texture.id(), TILE_SIZE)).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
            }
        }
    }

    // foreground
    if (state.display_layers & LAYER_FLAG_MAP_FG) != 0 {
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_layer_tile(map_data, LAYER_MAP_FG, x, y);
                if tile == 0xff || tile >= image.num_items { continue; }
                let tile_rect = get_tile_rect(x, y, state.zoom, canvas_rect.min + state.scroll).round_to_pixels(ui.pixels_per_point());
                Image::from_texture((texture.id(), TILE_SIZE)).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
            }
        }
    }

    // grid and border
    let stroke = egui::Stroke::new(1.0, Color32::BLACK);
    if (state.display_layers & LAYER_FLAG_GRID) != 0 {
        for y in 0..map_data.height+1 {
            let cy = (canvas_rect.min.y + y as f32 * zoomed_tile_size + state.scroll.y%zoomed_tile_size).round_to_pixels(ui.pixels_per_point());
            painter.hline(bg_rect.x_range(), cy, stroke);
        }
        for x in 0..map_data.width+1 {
            let cx = canvas_rect.min.x + x as f32 * zoomed_tile_size + state.scroll.x % zoomed_tile_size;
            painter.vline(cx, bg_rect.y_range(), stroke);
        }
    }
    let border_rect = bg_rect.expand2(Vec2::splat(-ui.pixels_per_point())).round_to_pixels(ui.pixels_per_point());
    painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

    // check zoom
    if let (true, Some(hover_pos)) = (
        response.contains_pointer(),
        ui.input(|i| i.pointer.hover_pos()),
    ) {
        let zoom_delta = ui.input(|i| i.zoom_delta());  // or use i.smooth_scroll_delta if CTRL key is should not be required?
        if zoom_delta != 1.0 {
            state.set_zoom(state.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, map_data);
        }
    }

    // check pan
    if response.dragged_by(egui::PointerButton::Middle) {
        state.scroll += response.drag_delta();
        state.clip_scroll(canvas_rect.size(), map_size);
    }

    // check click
    if let Some(pointer_pos) = response.interact_pointer_pos() {
        let tile = if response.dragged_by(egui::PointerButton::Primary) {
            state.left_draw_tile
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            state.right_draw_tile
        } else {
            0xffffffff
        };
        if tile != 0xffffffff && canvas_rect.contains(pointer_pos) {
            let tile_pos = ((pointer_pos - state.scroll - canvas_rect.min) / zoomed_tile_size).floor();
            set_layer_tile(map_data, tile_pos.x as i32, tile_pos.y as i32, state.edit_layer, tile);
        }
    }
}
