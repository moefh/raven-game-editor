use egui::{Vec2, Rect, Color32};

use crate::data_asset::{
    AssetList,
    World,
    Room,
    MapData,
    Tileset,
};
use crate::app::WindowContext;

use super::super::world_grid;
use super::TILE_SIZE;

pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    fn new(width: u32, height: u32) -> Self {
        Size {
            width,
            height,
        }
    }
}

pub struct RoomGridViewWidget;

impl RoomGridViewWidget {
    fn get_room_size(room: &Room, maps: &AssetList<MapData>) -> Size {
        room.maps.iter().fold(Size::new(0, 0), |size, room_map| {
            match maps.get(&room_map.map_id) {
                Some(map_data) => {
                    Size::new(
                        size.width.max(room_map.x as u32 + map_data.width),
                        size.height.max(room_map.y as u32 + map_data.height)
                    )
                }
                None => { size }
            }
        })
    }

    pub fn show(
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        world: &World,
        maps: &AssetList<MapData>,
        grid_store: &world_grid::WorldGridStore,
        region_index: usize,
    ) {
        let min_size = ui.available_size().max(Vec2::splat(50.0));
        let (response, painter) = ui.allocate_painter(min_size, egui::Sense::drag());
        let canvas_rect = response.rect;

        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0,0,0));

        let room_size = Self::get_room_size(room, maps);
        let grid_size = Size::new(room_size.width.div_ceil(wc.tiles_per_world_block), room_size.height.div_ceil(wc.tiles_per_world_block));
        let area_size = Size::new(
            grid_size.width * wc.tiles_per_world_block * Tileset::TILE_SIZE,
            grid_size.height * wc.tiles_per_world_block * Tileset::TILE_SIZE
        );
        let zoom = 0.8 * (canvas_rect.width() / area_size.width as f32).min(canvas_rect.height() / area_size.height as f32);
        let area_pos = canvas_rect.min + 0.5 * Vec2::new(
            (canvas_rect.width() - area_size.width as f32 * zoom).max(0.0),
            (canvas_rect.height() - area_size.height as f32 * zoom).max(0.0)
        );
        let area_rect = Rect::from_min_size(area_pos, zoom * Vec2::new(area_size.width as f32, area_size.height as f32));

        // grid
        let grid_stroke = egui::Stroke::new(1.0, Color32::from_rgb(192, 192, 192));
        painter.rect(area_rect, egui::CornerRadius::ZERO, Color32::from_rgb(128, 128, 128), grid_stroke, egui::StrokeKind::Middle);
        for y in 1..grid_size.height {
            let cy = area_rect.min.y + (y * wc.tiles_per_world_block) as f32 * TILE_SIZE * zoom;
            painter.hline(area_rect.x_range(), cy, grid_stroke);
        }
        for x in 1..grid_size.width {
            let cx = area_rect.min.x + (x * wc.tiles_per_world_block) as f32 * TILE_SIZE * zoom;
            painter.vline(cx, area_rect.y_range(), grid_stroke);
        }

        // maps
        let map_stroke = egui::Stroke::new(2.0, Color32::from_rgb(255, 64, 64));
        let map_fill = Color32::from_rgba_unmultiplied(255, 64, 64, 92);
        for room_map in room.maps.iter() {
            if let Some(map) = maps.get(&room_map.map_id) {
                let map_pos = zoom * TILE_SIZE * Vec2::new(room_map.x as f32, room_map.y as f32);
                let map_size = zoom * TILE_SIZE * Vec2::new(map.width as f32, map.height as f32);
                let map_rect = Rect::from_min_size(area_pos + map_pos, map_size);
                painter.rect(map_rect, egui::CornerRadius::ZERO, map_fill, map_stroke, egui::StrokeKind::Middle);
            }
        }

        // doors
        if let Some(grid) = grid_store.region_grids.get(region_index) &&
            let Some(region) = world.regions.get(region_index) &&
            let Some(room_info) = world_grid::RoomInfo::calculate(region, room, maps) {
                let door_zoom = zoom * room_info.width / room_info.block_width;
                for &door_index in grid.door_indices.iter() {
                    if let Some(door) = grid_store.doors.get(door_index) &&
                        door.room_id == Some(room.asset.id) &&
                        let Some(door_pos) = door.pos {
                            let door_pos = Vec2::new(
                                door_pos.x - grid.region_x - room_info.block_x,
                                door_pos.y - grid.region_y - room_info.block_y
                            );
                            painter.circle_filled(area_pos + door_zoom * door_pos, 5.0, Color32::from_rgb(0, 0, 255));
                        }
                }
            }
    }
}
