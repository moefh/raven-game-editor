use egui::{Vec2, Sense, Rect, Pos2, Image};

use crate::data_asset::{
    AssetList,
    Room,
    MapData,
    Tileset,
};
use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};

use super::super::MapLayer;
use super::{TILE_SIZE, get_map_layer_tile};

pub struct RoomViewWidget;

impl RoomViewWidget {
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

    fn get_room_size(room: &Room, maps: &AssetList<MapData>) -> Vec2 {
        let max = room.maps.iter().fold(Vec2::ZERO, |max, room_map| {
            match maps.get(&room_map.map_id) {
                Some(map_data) => max.max(Vec2::new((room_map.x as u32 + map_data.width) as f32, (room_map.y as u32 + map_data.height) as f32)),
                None => max,
            }
        });
        Vec2 {
            x: max.x * TILE_SIZE,
            y: max.y * TILE_SIZE,
        }
    }

    fn draw_map(ui: &mut egui::Ui, wc: &mut WindowContext, map_data: &MapData, map_pos: Pos2, zoom: f32, tileset: &Tileset) {
        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
        for y in 0..map_data.para_height {
            for x in 0..map_data.para_width {
                let tile = get_map_layer_tile(map_data, MapLayer::Parallax, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_tiles { continue; }
                let tile_rect = Self::get_tile_rect(x, y, zoom, map_pos);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, tile_rect);
            }
        }
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_tiles { continue; }
                let tile_rect = Self::get_tile_rect(x, y, zoom, map_pos);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, tile_rect);
            }
        }

        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_tiles { continue; }
                let tile_rect = Self::get_tile_rect(x, y, zoom, map_pos);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, tile_rect);
            }
        }
    }

    pub fn show(ui: &mut egui::Ui, wc: &mut WindowContext, room: &Room, maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) {
        let min_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let canvas_rect = response.rect;

        let room_size = Self::get_room_size(room, maps);

        if room_size.x == 0.0 || room_size.y == 0.0 {
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0,0,0));
            return;
        }

        let zoom = f32::min(canvas_rect.width() / room_size.x, canvas_rect.height() / room_size.y);

        let room_rect = egui::Rect {
            min: canvas_rect.min,
            max: canvas_rect.min + zoom * room_size,
        };
        painter.rect_filled(room_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0,0,0));

        for room_map in room.maps.iter() {
            if let Some(map) = maps.get(&room_map.map_id) && let Some(tileset) = tilesets.get(&map.tileset_id) {
                let pos = room_rect.min + Vec2::new(room_map.x as f32 * TILE_SIZE * zoom, room_map.y as f32 * TILE_SIZE * zoom);
                Self::draw_map(ui, wc, map, pos, zoom, tileset);
            }
        }
    }
}
