use egui::{
    emath,
    Vec2,
    Sense,
    Rect,
    Pos2,
};

use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::data_store::{
    DataAssetList,
    MapData,
    Tileset,
    TileAnimation,
};
use crate::misc::STATIC_IMAGES;

use super::{
    TILE_SIZE,
    FULL_UV,
    MapDisplay,
};
use super::super::{
    draw_bb_layer,
    draw_bg_layer,
    draw_fg_layer,
    WindowContext,
    DrawMapInfo,
};

pub struct MapTileChangePlacerWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub display: MapDisplay,
    dragging_tile_change: bool,
    drag_mouse_origin: Pos2,
    drag_tile_change_origin: Pos2,
    last_map_area_size: Vec2,
    tile_change_x: u32,
    tile_change_y: u32,
}

impl MapTileChangePlacerWidget {
    pub fn new() -> Self {
        let zoom = 0.75;
        MapTileChangePlacerWidget {
            zoom,
            scroll: Vec2::ZERO,
            display: MapDisplay::new(
                MapDisplay::FOREGROUND | MapDisplay::BACKGROUND | MapDisplay::BACK_BACKGROUND | MapDisplay::GRID
            ),
            tile_change_x: 0,
            tile_change_y: 0,
            dragging_tile_change: false,
            drag_mouse_origin: Pos2::ZERO,
            drag_tile_change_origin: Pos2::ZERO,
            last_map_area_size: Vec2::ZERO,
        }
    }

    pub fn clear_placement(&mut self) {
        //self.tile_change_x = 0;
        //self.tile_change_y = 0;
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

    fn handle_mouse_down(&mut self, resp: &egui::Response, mouse_pos: Pos2, map_data: &MapData, tile_change: usize) {
        if resp.drag_stopped() {
            self.dragging_tile_change = false;
            return;
        }
        let change = if let Some(change) = map_data.tile_changes.get(tile_change) {
            change
        } else {
            return;
        };

        if self.dragging_tile_change {
            if ! resp.dragged_by(egui::PointerButton::Primary) {
                self.dragging_tile_change = false;
                return;
            }
            let new_pos = self.drag_tile_change_origin + (mouse_pos - self.drag_mouse_origin);
            self.tile_change_x = (new_pos.x.round() as i32)
                .clamp(0, map_data.width.saturating_sub(change.width) as i32)
                .unsigned_abs();
            self.tile_change_y = (new_pos.y.round() as i32)
                .clamp(0, map_data.height.saturating_sub(change.height) as i32)
                .unsigned_abs();
            return;
        }

        let tile_change_pos = Pos2::new(self.tile_change_x as f32, self.tile_change_y as f32);
        let rect = Rect::from_min_size(tile_change_pos, Vec2::new(change.width as f32, change.height as f32));
        if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) && resp.drag_started() {
            self.dragging_tile_change = true;
            self.drag_tile_change_origin = tile_change_pos;
            self.drag_mouse_origin = mouse_pos;
        }
    }

    fn draw_tile_change(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        map_data: &MapData,
        tile_change: usize,
        tilesets: &DataAssetList<Tileset>,
        info: &DrawMapInfo,
    ) {
        if let Some(change) = map_data.tile_changes.get(tile_change) && let Some(tileset) = tilesets.get(&map_data.tileset_id) {
            for iy in 0..change.height {
                for ix in 0..change.width {
                    if let Some(tile) = change.tiles[(iy * change.width + ix) as usize] && tile != MapData::NO_TILE {
                        let x = ix + self.tile_change_x;
                        let y = iy + self.tile_change_y;
                        let (uv, texture) = if tile as u32 >= tileset.num_tiles {
                            (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        } else {
                            (tileset.get_item_uv(tile as u32), tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
                        };
                        let tile_rect = egui::Rect::from_min_max(
                            info.pos + info.zoom * TILE_SIZE * egui::Vec2::new(x as f32, y as f32),
                            info.pos + info.zoom * TILE_SIZE * egui::Vec2::new((x+1) as f32, (y+1) as f32)
                        );
                        if tile_rect.intersects(info.screen_rect) {
                            let image = egui::Image::from_texture((texture.id(), egui::Vec2::splat(TILE_SIZE))).uv(uv);
                            image.paint_at(ui, tile_rect)
                        }
                    }
                }
            }
        }
    }

    fn draw_tile_change_rect(painter: &egui::Painter, rect: Rect) {
        let outer_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        let inner_stroke = egui::Stroke::new(1.0, egui::Color32::RED);

        painter.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::from_rgba_unmultiplied(64, 128, 192, 192));
        painter.rect_stroke(rect, egui::CornerRadius::ZERO, outer_stroke, egui::StrokeKind::Outside);
        painter.rect_stroke(rect.expand(1.0), egui::CornerRadius::ZERO, inner_stroke, egui::StrokeKind::Outside);
        painter.rect_stroke(rect.expand(2.0), egui::CornerRadius::ZERO, outer_stroke, egui::StrokeKind::Outside);
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        map_data: &MapData,
        tile_change_index: usize,
        tilesets: &DataAssetList<Tileset>,
        tile_anims: &DataAssetList<TileAnimation>,
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
        let map_area_rect = if map_size.x >= canvas_rect.width() && map_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect::from_min_size(canvas_rect.min, map_size.min(canvas_rect.size()))
        };

        self.last_map_area_size = map_area_rect.size();

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), map_size);

        let canvas_to_map = emath::RectTransform::from_to(
            Rect::from_min_size(canvas_rect.min + self.scroll, map_size),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(map_data.width as f32, map_data.height as f32))
        );

        // ensure we don't draw outside the map area
        ui.shrink_clip_rect(canvas_rect);

        // draw background green
        painter.rect_filled(map_area_rect, egui::CornerRadius::ZERO, wc.settings.map_bg_color);

        let draw_info = DrawMapInfo {
            zoom: self.zoom,
            pos: canvas_rect.min + self.scroll,
            screen_rect: map_area_rect,
            animation_step: None,
        };

        if self.display.has_bits(MapDisplay::BACK_BACKGROUND) && map_data.has_bb {
            draw_bb_layer(ui, wc, map_data, tilesets, tile_anims, &draw_info, None);
        }
        if self.display.has_bits(MapDisplay::BACKGROUND) {
            draw_bg_layer(ui, wc, map_data, tilesets, tile_anims, &draw_info, None);
        }
        if self.display.has_bits(MapDisplay::FOREGROUND) {
            draw_fg_layer(ui, wc, map_data, tilesets, tile_anims, &draw_info, None);
        }

        if let Some(change) = map_data.tile_changes.get(tile_change_index) {
            let tile_change_pos = egui::Vec2::new(self.tile_change_x as f32, self.tile_change_y as f32);
            let sel_rect = Rect::from_min_size(
                canvas_rect.min + self.scroll + self.zoom * TILE_SIZE * tile_change_pos,
                self.zoom * TILE_SIZE * egui::Vec2::new(change.width as f32, change.height as f32),
            );
            Self::draw_tile_change_rect(&painter, sel_rect);
            self.draw_tile_change(ui, wc, map_data, tile_change_index, tilesets, &draw_info);
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, wc.settings.map_grid_color);
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

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // set cursor
        if response.contains_pointer() && response.hovered() {
            if keys_pressed.alt {
                response.ctx.set_cursor_icon(egui::CursorIcon::AllScroll);
            } else if keys_pressed.command {
                response.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, map_data);
            }
        }

        // check pan
        let middle_dragging = response.dragged_by(egui::PointerButton::Middle);
        if middle_dragging || (response.dragged() && keys_pressed.alt) {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), map_size);
        }

        // check click
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! keys_pressed.alt {
            let click_pos = canvas_to_map * pointer_pos;
            self.handle_mouse_down(&response, click_pos, map_data, tile_change_index);
        }
    }
}
