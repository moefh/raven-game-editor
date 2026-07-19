use crate::app::WindowContext;
use crate::data_asset::{
    World,
    WorldRegion,
};
use egui::{Vec2, Sense, Rect, Pos2, Color32};
use egui::emath;

use super::WidgetZoom;
use super::super::world_grid;

const BORDER_SIZE: Vec2 = Vec2::splat(1.0);

#[derive(Clone, Copy, PartialEq)]
enum DrawDoorType{
    Highlight,
    NoLink,
    SelfLink,
    IntraRegionLink,
    InterRegionLink,
}

pub struct WorldRegionEditorWidget {
    pub zoom: WidgetZoom,
    pub scroll: Vec2,
    pub selected_region: Option<usize>,
    pub selected_room: Option<u8>,
    pub hover_pos: Vec2,
    pub highlight_door_index: Option<usize>,
    tool_mouse_down: bool,
}

impl WorldRegionEditorWidget {
    const HIGHLIGHT_DOOR_DISTANCE: f32 = 20.0;

    pub fn new() -> Self {
        WorldRegionEditorWidget {
            zoom: WidgetZoom::FitToWindow,
            scroll: Vec2::ZERO,
            selected_room: None,
            selected_region: None,
            hover_pos: Vec2::ZERO,
            tool_mouse_down: false,
            highlight_door_index: None,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32, old_zoom: f32, center_delta: Vec2) {
        let zoom = zoom.max(2.0);
        let zoom_delta = zoom / old_zoom;
        self.zoom = WidgetZoom::Custom(zoom);
        self.scroll = center_delta - (center_delta - self.scroll) * zoom_delta;
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, region_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - region_size).min(Vec2::ZERO);
    }

    pub fn set_selected_region(&mut self, selected_region: Option<usize>) {
        if self.selected_region != selected_region {
            self.selected_region = selected_region;
            self.selected_room = None;
            self.highlight_door_index = None;
        }
    }

    pub fn get_selected_room(&self) -> Option<u8> {
        self.selected_room
    }

    pub fn set_selected_room(&mut self, room: Option<u8>) {
        self.selected_room = room;
    }

    fn set_region_block(&self, pos: Pos2, room: Option<u8>, region: &mut WorldRegion) {
        if pos.x < 0.0 || pos.y < 0.0 { return; }
        let x = pos.x.floor();
        let y = pos.y.floor();
        if x >= region.width as f32 || y >= region.height as f32 { return; }
        let x = x as usize;
        let y = y as usize;
        region.blocks[WorldRegion::BLOCK_STRIDE * y + x] = room;
    }

    fn get_region_block(&self, pos: Pos2, region: &mut WorldRegion) -> Option<u8> {
        if pos.x < 0.0 || pos.y < 0.0 { return None; }
        let x = pos.x.floor();
        let y = pos.y.floor();
        if x >= region.width as f32 || y >= region.height as f32 { return None; }
        let x = x as usize;
        let y = y as usize;
        region.blocks[WorldRegion::BLOCK_STRIDE * y + x]
    }

    fn handle_mouse(
        &mut self,
        pointer_pos: Pos2,
        response: &egui::Response,
        region: &mut WorldRegion,
    ) {
        let keys_pressed = response.ctx.input(|i| i.modifiers);

        if keys_pressed.ctrl {
            if response.dragged_by(egui::PointerButton::Primary) {
                let pick_room = self.get_region_block(pointer_pos, region);
                if pick_room.is_some() {
                    self.set_selected_room(pick_room);
                }
            }
            return;
        }

        if self.tool_mouse_down {
            if response.dragged_by(egui::PointerButton::Primary) {
                self.set_region_block(pointer_pos, self.selected_room, region);
            } else if response.dragged_by(egui::PointerButton::Secondary) {
                self.set_region_block(pointer_pos, None, region);
            }
        }
    }

    fn get_block_rect(x: u8, y: u8, zoom: f32, canvas_pos: Pos2) -> Rect {
        let pos = zoom * Vec2::new(x as f32, y as f32);
        Rect::from_min_size(canvas_pos + pos, zoom * Vec2::splat(1.0))
    }

    pub fn ensure_room_selection_is_valid(&mut self, region: &WorldRegion) {
        let num_rooms = region.rooms.len();
        if let Some(room_index) = self.get_selected_room() && room_index as usize >= num_rooms {
            if num_rooms > 0 {
                self.set_selected_room(Some(((num_rooms - 1) & 0xff) as u8));
            } else {
                self.set_selected_room(None);
            }
        }
    }

    fn draw_region_grid(&self, painter: &egui::Painter, region_pos: Pos2, zoom: f32, grid: &world_grid::Grid) {
        let stroke = egui::Stroke::new(2.0, Color32::WHITE);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let flags = grid.get_block_borders(x, y);
                if (flags & world_grid::Grid::BORDER_LEFT) != 0 {
                    let rx = region_pos.x + zoom * x as f32;
                    let ry = region_pos.y + zoom * y as f32;
                    painter.vline(rx, egui::Rangef::new(ry, ry+zoom), stroke);
                }
                if (flags & world_grid::Grid::BORDER_TOP) != 0 {
                    let rx = region_pos.x + zoom * x as f32;
                    let ry = region_pos.y + zoom * y as f32;
                    painter.hline(egui::Rangef::new(rx, rx+zoom), ry, stroke);
                }
            }
        }
    }

    fn draw_region_door(
        painter: &egui::Painter,
        door_index: usize,
        region_pos: Pos2,
        zoom: f32,
        grid_store: &world_grid::WorldGridStore,
        grid: &world_grid::Grid,
        draw_type: DrawDoorType,
    ) {
        if let Some(door) = grid_store.doors.get(door_index) && let Some(door_pos) = door.pos && let Some(region_index) = door.region_index {
            let draw_color = match draw_type {
                DrawDoorType::Highlight => {
                    Some(Color32::from_rgb(0xff, 0x00, 0xff))
                }
                DrawDoorType::NoLink => {
                    if door.dest_door_index.is_none() {
                        Some(Color32::from_rgb(0xff, 0x00, 0x00))
                    } else {
                        None
                    }
                }
                DrawDoorType::SelfLink => {
                    if door.dest_door_index == Some(door_index) {
                        Some(Color32::from_rgb(0xff, 0xff, 0x00))
                    } else {
                        None
                    }
                }
                DrawDoorType::IntraRegionLink => {
                    if door.dest_door_index.is_some() && door.dest_region_index == Some(region_index) {
                        Some(Color32::from_rgb(0x00, 0x80, 0x00))
                    } else {
                        None
                    }
                }
                DrawDoorType::InterRegionLink => {
                    if door.dest_door_index.is_some() && let Some(dest_region) = door.dest_region_index && dest_region != region_index {
                        Some(Color32::from_rgb(0x00, 0x80, 0xa0))
                    } else {
                        None
                    }
                }
            };
            if let Some(color) = draw_color {
                let door_pos = region_pos + zoom * egui::Vec2::new(door_pos.x - grid.region_x, door_pos.y - grid.region_y);
                if let Some(dest_door_index) = door.dest_door_index {
                    if let Some(dest_region_index) = door.dest_region_index && dest_region_index != region_index {
                        painter.circle_filled(door_pos, 6.0, Color32::from_rgb(0x00, 0xff, 0x00));
                    } else if let Some(dest_door) = grid_store.doors.get(dest_door_index) && let Some(dest_pos) = dest_door.pos {
                        let stroke = egui::Stroke { width: 2.0, color };
                        let dest_pos = region_pos + zoom * egui::Vec2::new(dest_pos.x - grid.region_x, dest_pos.y - grid.region_y);
                        painter.line_segment([door_pos, dest_pos], stroke);
                    }
                }
                painter.circle_filled(door_pos, 5.0, color);
            }
        }
    }

    fn draw_region(
        &self,
        painter: &egui::Painter,
        region_pos: Pos2,
        zoom: f32,
        grid_store: &world_grid::WorldGridStore,
        grid: &world_grid::Grid,
    ) {
        // room borders
        self.draw_region_grid(painter, region_pos, zoom, grid);

        // room doors
        for &door_index in grid.door_indices.iter() {
            Self::draw_region_door(painter, door_index, region_pos, zoom, grid_store, grid, DrawDoorType::InterRegionLink);
        }
        for &door_index in grid.door_indices.iter() {
            Self::draw_region_door(painter, door_index, region_pos, zoom, grid_store, grid, DrawDoorType::IntraRegionLink);
        }
        for &door_index in grid.door_indices.iter() {
            Self::draw_region_door(painter, door_index, region_pos, zoom, grid_store, grid, DrawDoorType::SelfLink);
        }
        for &door_index in grid.door_indices.iter() {
            Self::draw_region_door(painter, door_index, region_pos, zoom, grid_store, grid, DrawDoorType::NoLink);
        }
        if let Some(door_index) = self.highlight_door_index {
            Self::draw_region_door(painter, door_index, region_pos, zoom, grid_store, grid, DrawDoorType::Highlight);
        }
    }

    fn get_door_pos(door_index: usize, grid_store: &world_grid::WorldGridStore, grid: &world_grid::Grid) -> Option<Pos2> {
        if let Some(door) = grid_store.doors.get(door_index) && let Some(door_pos) = door.pos {
            Some(Pos2::new(door_pos.x - grid.region_x, door_pos.y - grid.region_y))
        } else {
            None
        }
    }

    fn get_closest_door(mouse_pos: Pos2, grid_store: &world_grid::WorldGridStore, grid: &world_grid::Grid) -> Option<(usize, f32)> {
        grid.door_indices.iter()
            .copied()
            .map(|index| {
                let dist = Self::get_door_pos(index, grid_store, grid).map(|p| (mouse_pos-p).length()).unwrap_or(f32::INFINITY);
                (index, dist)
            }).min_by(|(_, dist1), (_, dist2)| {
                dist1.total_cmp(dist2)
            })
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        world: &mut World,
        grid_store: &world_grid::WorldGridStore,
    ) {
        let (region, grid) =
            if let Some(region_index) = self.selected_region &&
            let Some(region) = world.regions.get_mut(region_index) &&
            let Some(grid) = grid_store.region_grids.get(region_index) {
                (region, grid)
            } else {
                ui.label("Invalid world grid");
                return;
            };

        self.ensure_room_selection_is_valid(region);

        let min_size = (Vec2::splat(50.0)).max(ui.available_size());
        let (response, mut painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let region_size = Vec2::new(region.width as f32, region.height as f32);
        let region_rect = Rect::from_min_size(Pos2::ZERO, region_size);
        let canvas_rect = response_rect.expand2(-BORDER_SIZE);

        let zoom = match self.zoom {
            WidgetZoom::Custom(zoom) => { zoom }
            WidgetZoom::FitToWindow => {
                let size = canvas_rect.size();
                let (zoomx, zoomy) = (size.x / region_size.x, size.y / region_size.y);
                f32::max(f32::min(zoomx, zoomy), 1.0)
            }
        };
        let zoomed_region_size = zoom * region_size;

        let region_area_rect = if zoomed_region_size.x >= canvas_rect.width() && zoomed_region_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            let size = zoomed_region_size.min(canvas_rect.size());
            Rect::from_min_size(canvas_rect.min + 0.5 * (canvas_rect.size() - size).max(Vec2::ZERO), size)
        };
        self.clip_scroll(canvas_rect.size(), zoomed_region_size);  // in case the window was resized
        let to_canvas = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, region_size),
            Rect::from_min_size(region_area_rect.min + self.scroll, zoomed_region_size),
        );
        let stroke = egui::Stroke::new(1.0, Color32::WHITE);
        let draw_rect = region_area_rect.expand2(BORDER_SIZE);
        painter.rect(draw_rect, egui::CornerRadius::ZERO, egui::Color32::BLACK, stroke, egui::StrokeKind::Middle);
        painter.shrink_clip_rect(draw_rect);
        ui.shrink_clip_rect(draw_rect);

        // room blocks
        let sel_room_color = Color32::from_rgb(0, 128, 192);
        let room_color = Color32::from_rgb(128, 128, 128);
        for y in 0..region.height {
            for x in 0..region.width {
                if let Some(block_room) = self.get_region_block(Pos2::new(x as f32, y as f32), region) {
                    let block_rect = Self::get_block_rect(x, y, zoom, region_area_rect.min + self.scroll);
                    let color = if Some(block_room) == self.selected_room {
                        sel_room_color
                    } else {
                        room_color
                    };
                    painter.rect_filled(block_rect, egui::CornerRadius::ZERO, color);
                }
            }
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, wc.settings.map_grid_color);
        for y in 0..region.height+1 {
            let cy = region_area_rect.min.y + y as f32 * zoom + self.scroll.y % zoom;
            painter.hline(region_area_rect.x_range(), cy, stroke);
        }
        for x in 0..region.width+1 {
            let cx = region_area_rect.min.x + x as f32 * zoom + self.scroll.x % zoom;
            painter.vline(cx, region_area_rect.y_range(), stroke);
        }
        let border_rect = region_area_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // region rooms and doors
        let region_pos = to_canvas.transform_pos(Pos2::ZERO);
        self.draw_region(&painter, region_pos, zoom, grid_store, grid);

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // set cursor
        if response.contains_pointer() && response.hovered() {
            if keys_pressed.alt {
                response.ctx.set_cursor_icon(egui::CursorIcon::AllScroll);
            } else if keys_pressed.ctrl {
                response.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
            }
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged() && keys_pressed.alt) {
            self.scroll += response.drag_delta();
            self.clip_scroll(region_area_rect.size(), to_canvas.transform_rect(region_rect).size());
        }

        // check click
        if response.drag_stopped() {
            self.tool_mouse_down = false;
        }
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! keys_pressed.alt {
            if response.drag_started() {
                self.tool_mouse_down = true;
            }
            let click_pos = to_canvas.inverse() * pointer_pos;
            self.handle_mouse(click_pos, &response, region);
        }

        // check hover
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let mouse_pos = to_canvas.inverse() * hover_pos;
            let closest_door = Self::get_closest_door(mouse_pos, grid_store, grid);
            self.highlight_door_index = if let Some((door_index, dist)) = closest_door && dist * zoom < Self::HIGHLIGHT_DOOR_DISTANCE {
                Some(door_index)
            } else {
                None
            };
        } else if self.highlight_door_index.is_some() {
            self.highlight_door_index = None;
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(zoom * zoom_delta, zoom, hover_pos - region_area_rect.min);
            }
            self.hover_pos = ((hover_pos - region_area_rect.min - self.scroll) / zoom).max(Vec2::ZERO);
        }
    }
}
