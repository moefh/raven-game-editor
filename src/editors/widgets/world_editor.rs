use egui::{Vec2, Sense, Rect, Pos2, Color32};
use egui::emath::RectTransform;

use crate::data_asset::{
    World,
    WorldRegion,
};
use crate::app::WindowContext;

use super::super::RectBorder;

const BORDER_SIZE: Vec2 = Vec2::splat(5.0);
const DRAG_BORDER_FUDGE_SIZE: f32 = 8.0;

struct RegionRect {
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
}

impl RegionRect {
    fn new(region: &WorldRegion) -> Self {
        RegionRect {
            x1: region.x as i16,
            y1: region.y as i16,
            x2: region.x as i16 + region.width as i16,
            y2: region.y as i16 + region.height as i16,
        }
    }

    fn set_left_border(&mut self, val: i16) {
        self.x1 = val.clamp(0, self.x2);
    }

    fn set_right_border(&mut self, val: i16) {
        self.x2 = val.clamp(self.x1 + 1, i16::MAX);
    }

    fn set_top_border(&mut self, val: i16) {
        self.y1 = val.clamp(0, self.y2);
    }

    fn set_bottom_border(&mut self, val: i16) {
        self.y2 = val.clamp(self.y1 + 1, i16::MAX);
    }

    fn apply_to_region(&self, region: &mut WorldRegion) {
        region.x = self.x1.clamp(0, u8::MAX as i16) as u8;
        region.y = self.y1.clamp(0, u8::MAX as i16) as u8;
        region.width = (self.x2 - self.x1).clamp(1, WorldRegion::MAX_WIDTH as i16) as u8;
        region.height = (self.y2 - self.y1).clamp(1, WorldRegion::MAX_HEIGHT as i16) as u8;
    }

    fn egui_rect(self) -> Rect {
        egui::Rect {
            min: Pos2::new(self.x1 as f32, self.y1 as f32),
            max: Pos2::new(self.x2 as f32, self.y2 as f32),
        }
    }
}

pub struct WorldEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub lock_regions: bool,
    selected_region_changed: bool,
    selected_region: Option<usize>,
    resize_border: Option<RectBorder>,
    dragging_region: bool,
    drag_region_origin: Pos2,
    drag_mouse_origin: Pos2,
    world_borders: super::WorldBorders,
}

impl WorldEditorWidget {
    pub fn new() -> Self {
        WorldEditorWidget {
            zoom: 20.0,
            scroll: Vec2::ZERO,
            selected_region: None,
            selected_region_changed: false,
            lock_regions: true,
            resize_border: None,
            dragging_region: false,
            drag_region_origin: Pos2::ZERO,
            drag_mouse_origin: Pos2::ZERO,
            world_borders: super::WorldBorders::new(),
        }
    }

    pub fn get_selected_region(&self) -> Option<usize> {
        self.selected_region
    }

    pub fn set_selected_region(&mut self, region_index: Option<usize>) {
        self.selected_region = region_index;
        self.selected_region_changed = true;
    }

    pub fn has_selected_region_changed(&self) -> bool {
        self.selected_region_changed
    }

    pub fn clear_selected_region_changed(&mut self) {
        self.selected_region_changed = false;
    }

    fn get_world_size(world: &World) -> Vec2 {
        world.regions.iter().fold(Vec2::ZERO, |max, region| {
            max.max(Vec2::new(region.x as f32 + region.width as f32, region.y as f32 + region.height as f32))
        })
    }

    fn get_region_rect(selected_region: Option<usize>, world: &World) -> Option<RegionRect> {
        if let Some(region_index) = selected_region && let Some(region) = world.regions.get(region_index) {
            Some(RegionRect::new(region))
        } else {
            None
        }
    }

    fn move_region(world: &mut World, region_index: usize, pos: Pos2) -> Option<bool> {
        if let Some(region) = world.regions.get_mut(region_index) {
            region.x = pos.x.round().clamp(0.0, u8::MAX as f32 - region.width as f32) as u8;
            region.y = pos.y.round().clamp(0.0, u8::MAX as f32 - region.height as f32) as u8;
            Some(true)
        } else {
            None
        }
    }

    fn get_rect_border(rect: Rect, pos: Pos2, zoom: f32) -> Option<RectBorder> {
        let fudge = DRAG_BORDER_FUDGE_SIZE / zoom;
        let corner_size = Vec2::splat(fudge);
        let horizontal_size = Vec2::new(rect.width(), fudge);
        let vertical_size = Vec2::new(fudge, rect.height());

        if Rect::from_center_size(rect.left_top(), corner_size).contains(pos) { return Some(RectBorder::TopLeft); }
        if Rect::from_center_size(rect.right_top(), corner_size).contains(pos) { return Some(RectBorder::TopRight); }
        if Rect::from_center_size(rect.right_bottom(), corner_size).contains(pos) { return Some(RectBorder::BottomRight); }
        if Rect::from_center_size(rect.left_bottom(), corner_size).contains(pos) { return Some(RectBorder::BottomLeft); }

        if Rect::from_center_size(rect.center_top(), horizontal_size).contains(pos) { return Some(RectBorder::Top); }
        if Rect::from_center_size(rect.center_bottom(), horizontal_size).contains(pos) { return Some(RectBorder::Bottom); }
        if Rect::from_center_size(rect.left_center(), vertical_size).contains(pos) { return Some(RectBorder::Left); }
        if Rect::from_center_size(rect.right_center(), vertical_size).contains(pos) { return Some(RectBorder::Right); }

        None
    }

    fn get_region_border(selected_region: Option<usize>, world: &World, pos: Pos2, zoom: f32) -> Option<RectBorder> {
        Self::get_rect_border(Self::get_region_rect(selected_region, world)?.egui_rect(), pos, zoom)
    }

    fn resize_start(&mut self, border: RectBorder, region_rect: Rect, mouse_pos: Pos2) {
        self.dragging_region = true;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = Some(border);
        self.drag_region_origin = match border {
            RectBorder::Top | RectBorder::Left | RectBorder::TopLeft => region_rect.min,
            RectBorder::Bottom | RectBorder::Right | RectBorder::BottomRight => region_rect.max,
            RectBorder::BottomLeft => Pos2::new(region_rect.min.x, region_rect.max.y),
            RectBorder::TopRight => Pos2::new(region_rect.max.x, region_rect.min.y),
        };
    }

    fn resize_move(&mut self, mouse_pos: Pos2, world: &mut World, border: RectBorder) -> bool {
        let new_pos = self.drag_region_origin + (mouse_pos - self.drag_mouse_origin);

        if self.dragging_region &&
            let Some(rect) = Self::get_region_rect(self.selected_region, world) &&
            let Some(region_index) = self.selected_region &&
            let Some(region) = world.regions.get_mut(region_index) {
                let mut rect = rect;
                let x = new_pos.x.clamp(0.0, u8::MAX as f32) as i16;
                let y = new_pos.y.clamp(0.0, u8::MAX as f32) as i16;
                match border {
                    RectBorder::Top         => { rect.set_top_border(y);    }
                    RectBorder::Left        => { rect.set_left_border(x);   }
                    RectBorder::Bottom      => { rect.set_bottom_border(y); }
                    RectBorder::Right       => { rect.set_right_border(x);  }
                    RectBorder::TopLeft     => { rect.set_top_border(y);    rect.set_left_border(x);  }
                    RectBorder::TopRight    => { rect.set_top_border(y);    rect.set_right_border(x); }
                    RectBorder::BottomRight => { rect.set_bottom_border(y); rect.set_right_border(x); }
                    RectBorder::BottomLeft  => { rect.set_bottom_border(y); rect.set_left_border(x);  }
                }
                rect.apply_to_region(region);
                true
            } else {
                false
            }
    }

    fn drag_start(&mut self, region_pos: Pos2, mouse_pos: Pos2) {
        self.dragging_region = true;
        self.drag_region_origin = region_pos;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = None;
    }

    fn drag_move(&mut self, mouse_pos: Pos2, world: &mut World) -> bool {
        if ! self.lock_regions && let Some(region_index) = self.selected_region {
            let new_pos = self.drag_region_origin + (mouse_pos - self.drag_mouse_origin);
            Self::move_region(world, region_index, new_pos).unwrap_or(false)
        } else {
            false
        }
    }

    fn drag_stop(&mut self) {
        self.dragging_region = false;
        self.resize_border = None;
    }

    fn handle_mouse_hover(&mut self, resp: &egui::Response, mouse_pos: Pos2, world: &mut World) {
        let keys_pressed = resp.ctx.input(|i| i.modifiers);
        if keys_pressed.alt {
            if resp.dragged() {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
        } else if let Some(border) = Self::get_region_border(self.selected_region, world, mouse_pos, self.zoom) {
            resp.ctx.set_cursor_icon(border.cursor());
        }
    }

    fn handle_mouse_down(&mut self, resp: &egui::Response, mouse_pos: Pos2, world: &mut World) {
        if resp.drag_stopped() {
            self.drag_stop();
            return;
        }

        if self.dragging_region && self.selected_region.is_some() {
            if ! resp.dragged_by(egui::PointerButton::Primary) {
                self.drag_stop();
                return;
            }
            if let Some(border) = self.resize_border {
                resp.ctx.set_cursor_icon(border.cursor());
                if self.resize_move(mouse_pos, world, border) {
                    return;
                }
            }
            if self.drag_move(mouse_pos, world) {
                return;
            }
            self.drag_stop();
        }

        // drag region border
        if resp.drag_started() &&
            resp.dragged_by(egui::PointerButton::Primary) &&
            let Some(border) = Self::get_region_border(self.selected_region, world, mouse_pos, self.zoom) &&
            let Some(rect) = Self::get_region_rect(self.selected_region, world) {
                self.resize_start(border, rect.egui_rect(), mouse_pos);
                return;
            }

        // click/drag selected region
        if let Some(rect) = Self::get_region_rect(self.selected_region, world) {
            let rect = rect.egui_rect();
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag trigger under the cursor
        for region_index in 0..world.regions.len() {
            let sel_region = Some(region_index);
            let rect = Self::get_region_rect(sel_region, world).map(|r| r.egui_rect()).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.set_selected_region(sel_region);
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // left-click nowhere deselects selected item
        if resp.dragged_by(egui::PointerButton::Primary) {
            self.set_selected_region(None);
        }
    }

    pub fn set_zoom(&mut self, zoom: f32, center_delta: Vec2) {
        let zoom = zoom.max(0.25);
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;
        self.scroll = center_delta - (center_delta - self.scroll) * zoom_delta;
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, trans_world_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - (trans_world_size + 2.0 * BORDER_SIZE)).min(Vec2::ZERO);
    }

    fn draw_selection_rect(painter: &egui::Painter, rect: Rect) {
        let outer_stroke = egui::Stroke::new(1.0, Color32::WHITE);
        let inner_stroke = egui::Stroke::new(1.0, Color32::RED);

        painter.rect_stroke(rect, egui::CornerRadius::ZERO, outer_stroke, egui::StrokeKind::Outside);
        painter.rect_stroke(rect.expand(1.0), egui::CornerRadius::ZERO, inner_stroke, egui::StrokeKind::Outside);
        painter.rect_stroke(rect.expand(2.0), egui::CornerRadius::ZERO, outer_stroke, egui::StrokeKind::Outside);
    }

    fn draw_outline_rect(painter: &egui::Painter, rect: Rect) {
        let outer_stroke = egui::Stroke::new(1.0, Color32::WHITE);
        let inner_stroke = egui::Stroke::new(1.0, Color32::BLUE);

        painter.rect_stroke(rect, egui::CornerRadius::ZERO, outer_stroke, egui::StrokeKind::Outside);
        painter.rect_stroke(rect.expand(1.0), egui::CornerRadius::ZERO, inner_stroke, egui::StrokeKind::Outside);
    }

    fn draw_region_blocks(&self, painter: &egui::Painter, rect: Rect, region: &WorldRegion) {
        if region.width == 0 || region.height == 0 { return; }
        let block_size = Vec2::new(
            rect.width() / region.width as f32,
            rect.height() / region.height as f32,
        );
        let bg_color = Color32::from_rgb(128, 128, 128);
        for y in 0..region.height {
            for x in 0..region.width {
                let block = region.blocks[y as usize * WorldRegion::BLOCK_STRIDE + x as usize];
                if block.is_some() {
                    let block_rect = Rect::from_min_size(rect.min + self.zoom * Vec2::new(x as f32, y as f32), block_size);
                    painter.rect_filled(block_rect, egui::CornerRadius::ZERO, bg_color);
                }
            }
        }
    }

    fn draw_world_borders(&mut self, painter: &egui::Painter, pos: Pos2, world: &World) {
        self.world_borders.update(world);

        let stroke = egui::Stroke::new(2.0, Color32::WHITE);
        for y in 0..self.world_borders.height {
            for x in 0..self.world_borders.width {
                let flags = self.world_borders.get_block_borders(x, y);
                if (flags & super::WorldBorders::BORDER_LEFT) != 0 {
                    let rx = pos.x + self.zoom * x as f32;
                    let ry = pos.y + self.zoom * y as f32;
                    painter.vline(rx, egui::Rangef::new(ry, ry+self.zoom), stroke);
                }
                if (flags & super::WorldBorders::BORDER_TOP) != 0 {
                    let rx = pos.x + self.zoom * x as f32;
                    let ry = pos.y + self.zoom * y as f32;
                    painter.hline(egui::Rangef::new(rx, rx+self.zoom), ry, stroke);
                }
            }
        }
    }

    pub fn ensure_room_selection_is_valid(&mut self, world: &World) {
        let num_regions = world.regions.len();
        if let Some(region_index) = self.get_selected_region() && region_index >= num_regions {
            if num_regions > 0 {
                self.set_selected_region(Some(num_regions - 1));
            } else {
                self.set_selected_region(None);
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, world: &mut World) {
        self.ensure_room_selection_is_valid(world);

        let min_size = ui.available_size();
        let (response, mut painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let world_size = Self::get_world_size(world);
        let world_rect = Rect::from_min_size(Pos2::ZERO, world_size);
        let canvas_rect = response_rect.expand2(-Vec2::splat(1.0));

        self.clip_scroll(canvas_rect.size(), world_rect.size() * self.zoom); // in case the window was resized
        let to_canvas = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, world_size),
            Rect::from_min_size(canvas_rect.min + BORDER_SIZE + self.scroll, world_size * self.zoom),
        );
        let bg_rect = Rect {
            min: response_rect.min,
            max: response_rect.max.min(response_rect.min + world_size * self.zoom + Vec2::splat(2.0) + 2.0 * BORDER_SIZE),
        };
        painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0, 0));
        let stroke = egui::Stroke::new(1.0, Color32::WHITE);
        painter.rect_stroke(bg_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
        painter.shrink_clip_rect(canvas_rect);
        ui.shrink_clip_rect(canvas_rect);
        Self::draw_outline_rect(&painter, canvas_rect);

        if canvas_rect.width() == 0.0 || canvas_rect.height() == 0.0 || world_rect.width() == 0.0 || world_rect.height() == 0.0 {
            return; // nothing to do!
        }

        // draw world
        for (region_index, region) in world.regions.iter().enumerate() {
            if let Some(rect) = Self::get_region_rect(Some(region_index), world) {
                let rect = to_canvas.transform_rect(rect.egui_rect());
                self.draw_region_blocks(&painter, rect, region);
                Self::draw_outline_rect(&painter, rect);
            }
        }
        self.draw_world_borders(&painter, to_canvas.transform_pos(Pos2::ZERO), world);

        // outline selected region
        if let Some(rect) = Self::get_region_rect(self.selected_region, world) {
            Self::draw_selection_rect(&painter, to_canvas.transform_rect(rect.egui_rect()));
        }

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // check hover
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let mouse_pos = to_canvas.inverse() * hover_pos;
            self.handle_mouse_hover(&response, mouse_pos, world);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || keys_pressed.alt {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), to_canvas.transform_rect(world_rect).size());
        }

        // check click
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! (keys_pressed.alt || keys_pressed.ctrl) {
            let click_pos = to_canvas.inverse() * pointer_pos;
            self.handle_mouse_down(&response, click_pos, world);
        }

        // check zoom (must be last)
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let zoom_delta = if keys_pressed.ctrl && response.dragged_by(egui::PointerButton::Primary) {
                (response.drag_delta().y * -0.01).exp()
            } else {
                ui.input(|i| i.zoom_delta())
            };
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, hover_pos - canvas_rect.min);
            }
        }
    }
}
