use egui::{Vec2, Sense, Rect, Pos2, Image};

use crate::data_asset::{
    MapData,
    Tileset,
};
use crate::app::WindowContext;
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::super::TileGrid;
use super::TILE_SIZE;

pub enum TileGridEditorAction {
    None,
    PickLeftTile(Option<u32>),
    PickRightTile(Option<u32>),
    SetTile,
}

pub struct TileGridEditorWidget {
    pub left_selected_tile: Option<u32>,
    pub right_selected_tile: Option<u32>
}

impl TileGridEditorWidget {
    const BORDER: f32 = 10.0;

    pub fn new() -> Self {
        TileGridEditorWidget {
            left_selected_tile: None,
            right_selected_tile: None,
        }
    }

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

    fn draw_tiles(ui: &mut egui::Ui, wc: &mut WindowContext, tile_grid: &TileGrid, tileset: &Tileset, zoom: f32, pos: Pos2) {
        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        for y in 0..tile_grid.height {
            for x in 0..tile_grid.width {
                let tile = tile_grid.get_tile(x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_tiles { continue; }
                let tile_rect = Self::get_tile_rect(x, y, zoom, pos);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, tile_rect);
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, tile_grid: &mut TileGrid, tileset: &Tileset) -> TileGridEditorAction {
        let min_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let canvas_rect = response.rect;

        if tile_grid.width == 0 || tile_grid.height == 0 {
            painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, egui::Color32::BLACK);
            return TileGridEditorAction::None;
        }

        let map_size = egui::Vec2::new(tile_grid.width as f32 * TILE_SIZE, tile_grid.height as f32 * TILE_SIZE);
        let zoom = f32::min(
            (canvas_rect.width()  - 2.0*Self::BORDER) / map_size.x,
            (canvas_rect.height() - 2.0*Self::BORDER) / map_size.y,
        );
        let map_pos = canvas_rect.min + 0.5 * (canvas_rect.size() - zoom * map_size);
        let map_rect = egui::Rect::from_min_size(map_pos, zoom * map_size);

        // clear map BG
        painter.rect_filled(map_rect, egui::CornerRadius::ZERO, wc.settings.image_bg_color);

        // tiles
        Self::draw_tiles(ui, wc, tile_grid, tileset, zoom, map_pos);

        // border and grid
        let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        let border_rect = map_rect.expand2(Vec2::splat(1.0));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
        for y in 0..tile_grid.height {
            painter.hline(border_rect.x_range(), map_pos.y + y as f32 * zoom * TILE_SIZE, stroke);
        }
        for x in 0..tile_grid.width {
            painter.vline(map_pos.x + x as f32 * zoom * TILE_SIZE, border_rect.y_range(), stroke);
        }

        // click
        let keys_pressed = ui.ctx().input(|i| i.modifiers);
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! keys_pressed.alt {
            let pos = (pointer_pos - map_pos) / TILE_SIZE / zoom;
            let x = pos.x.floor().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
            let y = pos.y.floor().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
            if x >= 0 && y >= 0 && (x as u32) < tile_grid.width && (y as u32) < tile_grid.height {
                let x = x as u32;
                let y = y as u32;
                if keys_pressed.ctrl {
                    let tile = tile_grid.get_tile(x, y);
                    let tile = if tile == MapData::NO_TILE { None } else { Some(tile as u32) };
                    if response.dragged_by(egui::PointerButton::Primary) {
                        self.left_selected_tile = tile;
                        return TileGridEditorAction::PickLeftTile(tile);
                    }
                    if response.dragged_by(egui::PointerButton::Secondary) {
                        self.right_selected_tile = tile;
                        return TileGridEditorAction::PickRightTile(tile);
                    }
                } else {
                    let tile = if response.dragged_by(egui::PointerButton::Primary) {
                        Ok(self.left_selected_tile)
                    } else if response.dragged_by(egui::PointerButton::Secondary) {
                        Ok(self.right_selected_tile)
                    } else {
                        Err(())
                    };
                    if let Ok(tile) = tile {
                        if let Some(tile) = tile {
                            tile_grid.set_tile(x, y, (tile & 0xff) as u8);
                        } else {
                            tile_grid.set_tile(x, y, MapData::NO_TILE);
                        }
                    }
                    return TileGridEditorAction::SetTile;
                }
            }
        }
        TileGridEditorAction::None
    }
}
