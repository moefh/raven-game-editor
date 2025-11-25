use egui::{Vec2, Sense, Rect, Pos2, Image};

use crate::data_asset::{MapData, Tileset};
use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};

use super::{MapLayer, TILE_SIZE, get_map_layer_tile};

fn get_tile_rect(x: u32, y: u32, zoom: f32, canvas_pos: Pos2) -> Rect {
    let pos = Vec2 {
        x: x as f32 * zoom * TILE_SIZE,
        y: y as f32 * zoom * TILE_SIZE,
    };
    Rect {
        min: canvas_pos + pos,
        max: canvas_pos + pos + zoom * Vec2::splat(TILE_SIZE),
    }
}

pub fn map_view(ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &MapData, tileset: &Tileset) {
    let min_size = ui.available_size();
    let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
    let canvas_rect = response.rect;

    if map_data.width == 0 || map_data.height == 0 {
        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0,0,0));
        return;
    }

    let map_size = egui::Vec2::new(map_data.width as f32 * TILE_SIZE, map_data.height as f32 * TILE_SIZE);
    let zoom = f32::min(canvas_rect.width() / map_size.x, canvas_rect.height() / map_size.y);

    let map_rect = egui::Rect {
        min: canvas_rect.min,
        max: canvas_rect.min + zoom * map_size,
    };
    painter.rect_filled(map_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0,0,0));

    let image = ImageCollection::from_asset(tileset);

    let texture = image.texture(wc.tex_man, wc.egui.ctx, tileset, TextureSlot::Opaque);
    for y in 0..map_data.bg_height {
        for x in 0..map_data.bg_width {
            let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let tile_rect = get_tile_rect(x, y, zoom, map_rect.min);
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
        }
    }

    let texture = image.texture(wc.tex_man, wc.egui.ctx, tileset, TextureSlot::Transparent);
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let tile_rect = get_tile_rect(x, y, zoom, map_rect.min);
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
        }
    }
}
