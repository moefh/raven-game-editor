use crate::app::WindowContext;
use crate::data_asset::WorldRegion;
use egui::{Vec2, Sense, Rect, Pos2, Color32};
use egui::emath;

pub struct WorldRegionEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub selected_room: Option<u8>,
    pub selected_room_changed: bool,
    pub hover_pos: Vec2,
}

impl WorldRegionEditorWidget {
    pub fn new() -> Self {
        let zoom = 10.0;
        WorldRegionEditorWidget {
            zoom,
            scroll: Vec2::ZERO,
            selected_room: None,
            selected_room_changed: false,
            hover_pos: Vec2::ZERO,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32, canvas_size: Vec2, zoom_center: Vec2, region: &WorldRegion) {
        let zoom = zoom.max(2.0);
        let region_size = Vec2 {
            x: region.width as f32 * zoom,
            y: region.height as f32 * zoom,
        };
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;

        self.scroll = zoom_center - (zoom_center - self.scroll) * zoom_delta;
        self.clip_scroll(canvas_size, region_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, region_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - region_size).min(Vec2::ZERO);
    }

    pub fn get_selected_room(&self) -> Option<u8> {
        self.selected_room
    }

    pub fn set_selected_room(&mut self, room: Option<u8>) {
        self.selected_room = room;
        self.selected_room_changed = true;
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

    fn handle_mouse(&mut self, pointer_pos: Pos2, response: &egui::Response,
                    region: &mut WorldRegion, canvas_to_region: &emath::RectTransform) {
        let keys_pressed = response.ctx.input(|i| i.modifiers);

        if keys_pressed.ctrl {
            if response.dragged_by(egui::PointerButton::Primary) {
                let pick_room = self.get_region_block(canvas_to_region * pointer_pos, region);
                if pick_room.is_some() {
                    self.set_selected_room(pick_room);
                }
            }
        } else if response.dragged_by(egui::PointerButton::Primary) {
            self.set_region_block(canvas_to_region * pointer_pos, self.selected_room, region);
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            self.set_region_block(canvas_to_region * pointer_pos, None, region);
        }
    }

    fn get_block_rect(x: u8, y: u8, zoom: f32, canvas_pos: Pos2) -> Rect {
        let pos = Vec2 {
            x: x as f32 * zoom,
            y: y as f32 * zoom,
        };
        Rect {
            min: canvas_pos + pos,
            max: canvas_pos + pos + zoom * Vec2::splat(1.0),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, region: &mut WorldRegion) {
        let min_size = (self.zoom * Vec2::splat(1.0)).max(ui.available_size());
        let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let canvas_rect = Rect {
            min: response_rect.min.floor(),
            max: response_rect.max.floor(),
        };
        let region_size = Vec2 {
            x: region.width as f32 * self.zoom,
            y: region.height as f32 * self.zoom,
        };
        let region_area_rect = if region_size.x >= canvas_rect.width() && region_size.y >= canvas_rect.height() {
            canvas_rect
        } else {
            Rect::from_min_size(canvas_rect.min, region_size.min(canvas_rect.size()))
        };

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), region_size);

        let canvas_to_region = emath::RectTransform::from_to(
            Rect { min: canvas_rect.min + self.scroll, max: canvas_rect.min + region_size + self.scroll },
            Rect { min: Pos2::ZERO, max: Pos2::new(region.width as f32, region.height as f32) }
        );

        // ensure we don't draw outside the region area
        ui.shrink_clip_rect(canvas_rect);

        // draw background
        painter.rect_filled(region_area_rect, egui::CornerRadius::ZERO, Color32::BLACK);

        // background
        for y in 0..region.height {
            for x in 0..region.width {
                if let Some(block_room) = self.get_region_block(Pos2::new(x as f32, y as f32), region) {
                    let block_rect = Self::get_block_rect(x, y, self.zoom, canvas_rect.min + self.scroll);
                    let color = if Some(block_room) == self.selected_room {
                        Color32::RED
                    } else {
                        Color32::WHITE
                    };
                    painter.rect_filled(block_rect, egui::CornerRadius::ZERO, color);
                }
            }
        }

        // grid and border
        let stroke = egui::Stroke::new(1.0, wc.settings.map_grid_color);
        for y in 0..region.height+1 {
            let cy = canvas_rect.min.y + y as f32 * self.zoom + self.scroll.y % self.zoom;
            painter.hline(region_area_rect.x_range(), cy, stroke);
        }
        for x in 0..region.width+1 {
            let cx = canvas_rect.min.x + x as f32 * self.zoom + self.scroll.x % self.zoom;
            painter.vline(cx, region_area_rect.y_range(), stroke);
        }
        let border_rect = region_area_rect.expand2(Vec2::splat(-ui.pixels_per_point()));
        painter.rect_stroke(border_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Outside);

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // set move cursor if ALT is pressed
        if response.contains_pointer() && response.hovered() && keys_pressed.alt {
            if response.dragged() {
                response.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                response.ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, canvas_rect.size(), hover_pos - canvas_rect.min, region);
            }
            self.hover_pos = ((hover_pos - canvas_rect.min - self.scroll) / self.zoom).max(Vec2::ZERO);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || keys_pressed.alt {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), region_size);
        }

        // check click
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! keys_pressed.alt {
            self.handle_mouse(pointer_pos, &response, region, &canvas_to_region);
        }
    }
}
