use egui::{
    Vec2,
    Sense,
    Rect,
    Image,
};

use crate::data_asset::{
    Tileset,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::TILE_SIZE;

use super::super::{
    WindowContext,
};

pub struct TilePickerPopupWidget {
    pub open: bool,
    egui_id: egui::Id,
    close_on_pick: bool,
}

impl TilePickerPopupWidget {
    pub fn new(egui_id: egui::Id) -> Self{
        TilePickerPopupWidget {
            egui_id,
            close_on_pick: true,
            open: false,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    fn draw_tileset(
        painter: &egui::Painter,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        rect: Rect,
        dims: (i32, i32),
        tileset: &Tileset,
        sel_tile: Option<u32>
    ) {
        // tiles
        let tile_size = Vec2::new(rect.width() / (dims.0 as f32), rect.height() / (dims.1 as f32));
        for y in 0..dims.1 {
            for x in 0..dims.0 {
                let tile_rect = Rect::from_min_size(rect.min + tile_size * Vec2::new(x as f32, y as f32), tile_size);
                let tile = ((y * dims.0 + x).abs() & 0xff) as u32;
                if tile >= tileset.num_tiles { break; }

                let uv = tileset.get_item_uv(tile);
                let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(uv).paint_at(ui, tile_rect);
            }
        }

        // grid
        let stroke = egui::Stroke::new(1.0, wc.settings.map_grid_color);
        for y in 0..dims.1+1 {
            let cy = rect.min.y + y as f32 * tile_size.y;
            painter.hline(rect.x_range(), cy, stroke);
        }
        for x in 0..dims.0+1 {
            let cx = rect.min.x + x as f32 * tile_size.x;
            painter.vline(cx, rect.y_range(), stroke);
        }

        // selection rectangle
        if let Some(tile) = sel_tile {
            let x = tile as i32 % dims.0;
            let y = tile as i32 / dims.0;
            let tile_rect = Rect::from_min_size(rect.min + tile_size * Vec2::new(x as f32, y as f32), tile_size);
            let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 255));
            painter.rect_stroke(tile_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, response: &egui::Response, tileset: &Tileset, pick_tile: &mut Option<u32>) -> bool {
        if ! self.open { return false; }
        let tile_size = Vec2::splat(TILE_SIZE * (wc.settings.tile_picker_popup_zoom as f32 / 100.0));
        let mut picked = false;
        egui::containers::Popup::menu(response)
            .id(self.egui_id)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.horizontal(|ui| {
                    let dim_y = ((tileset.num_tiles as f32 / 2.0).sqrt()).ceil() as i32;
                    let dims = (tileset.num_tiles.div_ceil(dim_y as u32) as i32, dim_y);
                    let size = Vec2::new(dims.0 as f32 * tile_size.x, dims.1 as f32 * tile_size.y);
                    let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
                    Self::draw_tileset(&painter, ui, wc, response.rect, dims, tileset, *pick_tile);
                    if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                        let x = ((pos.x - response.rect.min.x) / tile_size.x).floor() as i32;
                        let y = ((pos.y - response.rect.min.y) / tile_size.y).floor() as i32;
                        let index = y * dims.0 + x;
                        if index >= 0 && index < tileset.num_tiles as i32 {
                            *pick_tile = Some(index.unsigned_abs());
                            picked = true;
                        }
                    }
                    if (self.close_on_pick && response.clicked()) || response.double_clicked() {
                        self.open = false;
                    }
                });
            });
        picked
    }

    pub fn show_anchor(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        label: impl AsRef<str>,
        tileset: &Tileset,
        pick_tile: Option<u32>
    ) -> Option<u32> {
        let response = ui.button(label.as_ref());
        if response.clicked() {
            self.open();
        }
        let mut tile = pick_tile;
        if self.show(wc, &response, tileset, &mut tile) {
            tile
        } else {
            None
        }
    }
}
