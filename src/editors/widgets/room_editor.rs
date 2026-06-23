use egui::{Vec2, Sense, Rect, Pos2, Image, Color32};
use egui::emath::RectTransform;

use crate::data_asset::{
    AssetList,
    Room,
    RoomMap,
    RoomTrigger,
    RoomTriggerType,
    MapData,
    Tileset,
    Sprite,
};
use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};

use super::{TILE_SIZE, SCREEN_SIZE, get_map_layer_tile};
use super::super::{MapLayer, RectBorder};
use super::super::room::{RoomEditorAssetLists, RoomItemRef};

const BORDER_SIZE: Vec2 = Vec2::splat(5.0);
const DRAG_BORDER_FUDGE_SIZE: f32 = 8.0;

struct GridAlign {
    size: u16,
    align: bool,
}

impl GridAlign {
    fn new(size: u16) -> Self {
        GridAlign {
            size,
            align: false,
        }
    }

    fn align_i16(&self, val: i16) -> i16 {
        if self.align {
            let size = self.size as i16;
            (val + size/2) / size * size
        } else {
            val
        }
    }

    fn align_i32(&self, val: i32) -> i32 {
        if self.align {
            let size = self.size as i32;
            (val + size/2) / size * size
        } else {
            val
        }
    }
}

struct ScreenPos {
    x: u16,
    y: u16,
}

impl ScreenPos {
    fn get_rect(&self) -> Rect {
        egui::Rect::from_min_size(egui::Pos2::new(self.x as f32, self.y as f32), SCREEN_SIZE)
    }
}

struct TriggerPos {
    x: i32,
    y: i32,
}

impl TriggerPos {
    fn from_pos2(pos: Pos2) -> Self {
        TriggerPos {
            x: pos.x.clamp(i32::MIN as f32, i32::MAX as f32) as i32,
            y: pos.y.clamp(i32::MIN as f32, i32::MAX as f32) as i32,
        }
    }
}

struct TriggerRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    resizable: bool,
}

impl TriggerRect {
    fn new(x: i16, y: i16, width: u16, height: u16, resizable: bool) -> Self {
        TriggerRect {
            x1: x as i32,
            y1: y as i32,
            x2: x as i32 + width as i32,
            y2: y as i32 + height as i32,
            resizable,
        }
    }

    fn resizable(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self::new(x, y, width, height, true)
    }

    fn rigid(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self::new(x, y, width, height, false)
    }

    fn from_trigger(trigger: &RoomTrigger, assets: &RoomEditorAssetLists) -> Self {
        match trigger.trigger_type {
            RoomTriggerType::Trap { width, height, .. } => {
                TriggerRect::resizable(trigger.x, trigger.y, width, height)
            }
            RoomTriggerType::EnemySpawn { animation_id } => {
                assets.animations.get(&animation_id)
                    .and_then(|animation| { assets.sprites.get(&animation.sprite_id) })
                    .map(|sprite| { TriggerRect::rigid(trigger.x, trigger.y,
                                                           (sprite.width & 0xffff) as u16,
                                                           (sprite.height & 0xffff) as u16) })
                    .unwrap_or_else(|| TriggerRect::rigid(trigger.x, trigger.y, 64, 64))
            }
            RoomTriggerType::Door {..} => {
                TriggerRect::rigid(trigger.x, trigger.y, 16, 64)
            }
            RoomTriggerType::PlayerSpawn {..} |
            RoomTriggerType::Unknown {..} => {
                TriggerRect::rigid(trigger.x, trigger.y, 64, 64)
            }
        }
    }

    fn set_left_border(&mut self, val: i32) {
        self.x1 = val.clamp(-256, self.x2);
    }

    fn set_right_border(&mut self, val: i32) {
        self.x2 = val.clamp(self.x1, u16::MAX as i32);
    }

    fn set_top_border(&mut self, val: i32) {
        self.y1 = val.clamp(-256, self.y2);
    }

    fn set_bottom_border(&mut self, val: i32) {
        self.y2 = val.clamp(self.y1, u16::MAX as i32);
    }

    fn apply_to_trigger(&self, trigger: &mut RoomTrigger) {
        trigger.x = self.x1.clamp(-256, i16::MAX as i32) as i16;
        trigger.y = self.y1.clamp(-256, i16::MAX as i32) as i16;
        match &mut trigger.trigger_type {
            RoomTriggerType::Trap { width, height, .. } => {
                *width = (self.x2 - self.x1).clamp(0, u16::MAX as i32) as u16;
                *height = (self.y2 - self.y1).clamp(0, u16::MAX as i32) as u16;
            }
            RoomTriggerType::Door {..} |
            RoomTriggerType::EnemySpawn {..} |
            RoomTriggerType::PlayerSpawn {..} |
            RoomTriggerType::Unknown {..} => {
                // not resizable
            }
        }
    }

    fn egui_rect(self) -> Rect {
        egui::Rect {
            min: Pos2::new(self.x1 as f32, self.y1 as f32),
            max: Pos2::new(self.x2 as f32, self.y2 as f32),
        }
    }
}

pub struct RoomEditorWidget {
    pub zoom: f32,
    pub scroll: Vec2,
    pub lock_maps: bool,
    pub show_screen: bool,
    screen_pos: ScreenPos,
    selected_item_changed: bool,
    selected_item: RoomItemRef,
    resize_border: Option<RectBorder>,
    dragging_item: bool,
    drag_item_origin: Pos2,
    drag_mouse_origin: Pos2,
    grid: GridAlign,
}

impl RoomEditorWidget {
    pub fn new() -> Self {
        RoomEditorWidget {
            zoom: 0.5,
            scroll: Vec2::ZERO,
            selected_item: RoomItemRef::None,
            selected_item_changed: false,
            lock_maps: true,
            show_screen: false,
            screen_pos: ScreenPos { x: 0, y: 0 },
            resize_border: None,
            dragging_item: false,
            drag_item_origin: Pos2::ZERO,
            drag_mouse_origin: Pos2::ZERO,
            grid: GridAlign::new(Tileset::TILE_SIZE as u16),
        }
    }

    pub fn get_selected_item(&self) -> RoomItemRef {
        self.selected_item
    }

    pub fn set_selected_item(&mut self, item: RoomItemRef) {
        self.selected_item = item;
        self.selected_item_changed = true;
    }

    pub fn has_selected_item_changed(&self) -> bool {
        self.selected_item_changed
    }

    pub fn clear_selected_item_changed(&mut self) {
        self.selected_item_changed = false;
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

    fn get_tile_rect(x: u32, y: u32, map_pos: Pos2) -> Rect {
        let tile_pos = Vec2 {
            x: TILE_SIZE * (x as f32),
            y: TILE_SIZE * (y as f32),
        };
        Rect {
            min: map_pos + tile_pos,
            max: map_pos + tile_pos + Vec2::splat(TILE_SIZE),
        }
    }

    fn get_map_rect(room_map: &RoomMap, map_data: &MapData) -> Rect {
        let map_pos = Pos2::new(room_map.x as f32, room_map.y as f32);
        let map_size = Vec2::new(map_data.width as f32, map_data.height as f32);
        egui::Rect {
            min: TILE_SIZE * map_pos,
            max: TILE_SIZE * (map_pos + map_size),
        }
    }

    fn get_trigger_rect(trigger: &RoomTrigger, assets: &RoomEditorAssetLists) -> Rect {
        TriggerRect::from_trigger(trigger, assets).egui_rect()
    }

    fn get_item_rect(item: RoomItemRef, room: &Room, assets: &RoomEditorAssetLists, screen_pos: &ScreenPos) -> Option<Rect> {
        match item {
            RoomItemRef::None => None,

            RoomItemRef::Screen => {
                Some(screen_pos.get_rect())
            }

            RoomItemRef::Map(map_index) => {
                let room_map = room.maps.get(map_index)?;
                let map_data = assets.maps.get(&room_map.map_id)?;
                Some(Self::get_map_rect(room_map, map_data))
            }

            RoomItemRef::Trigger(trg_index) => {
                let trigger = room.triggers.get(trg_index)?;
                Some(Self::get_trigger_rect(trigger, assets))
            }
        }
    }

    fn move_item(item: &mut RoomItemRef, pos: Pos2, room: &mut Room, screen_pos: &mut ScreenPos,
                 lock_maps: bool, grid: &GridAlign) -> Option<bool> {
        match item {
            RoomItemRef::None => { None }

            RoomItemRef::Screen => {
                screen_pos.x = pos.x.round().clamp(0.0, u16::MAX as f32) as u16;
                screen_pos.y = pos.y.round().clamp(0.0, u16::MAX as f32) as u16;
                Some(true)
            }

            RoomItemRef::Map(map_index) => {
                if lock_maps {
                    None
                } else {
                    let room_map = room.maps.get_mut(*map_index)?;
                    room_map.x = (pos.x / TILE_SIZE).round().clamp(0.0, 1024.0) as u16;
                    room_map.y = (pos.y / TILE_SIZE).round().clamp(0.0, 1024.0) as u16;
                    Some(true)
                }
            }

            RoomItemRef::Trigger(trg_index) => {
                let trigger = room.triggers.get_mut(*trg_index)?;
                trigger.x = grid.align_i16(pos.x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16);
                trigger.y = grid.align_i16(pos.y.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16);
                Some(true)
            }
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

    fn get_trigger_border(item: RoomItemRef, room: &mut Room, pos: Pos2, zoom: f32, assets: &RoomEditorAssetLists) -> Option<RectBorder> {
        if let RoomItemRef::Trigger(trg_index) = item {
            let trigger = room.triggers.get(trg_index)?;
            let rect = TriggerRect::from_trigger(trigger, assets);
            if rect.resizable {
                Self::get_rect_border(rect.egui_rect(), pos, zoom)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn resize_start(&mut self, border: RectBorder, item_rect: Rect, mouse_pos: Pos2) {
        self.dragging_item = true;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = Some(border);
        self.drag_item_origin = match border {
            RectBorder::Top | RectBorder::Left | RectBorder::TopLeft => item_rect.min,
            RectBorder::Bottom | RectBorder::Right | RectBorder::BottomRight => item_rect.max,
            RectBorder::BottomLeft => Pos2::new(item_rect.min.x, item_rect.max.y),
            RectBorder::TopRight => Pos2::new(item_rect.max.x, item_rect.min.y),
        };
    }

    fn resize_move(&mut self, mouse_pos: Pos2, room: &mut Room, border: RectBorder, assets: &RoomEditorAssetLists) -> bool {
        let new_pos = TriggerPos::from_pos2(self.drag_item_origin + (mouse_pos - self.drag_mouse_origin));

        if self.dragging_item &&
            let RoomItemRef::Trigger(index) = self.selected_item &&
            let Some(trigger) = room.triggers.get_mut(index) {
                let mut rect = TriggerRect::from_trigger(trigger, assets);
                if ! rect.resizable { return false; }
                let x = self.grid.align_i32(new_pos.x);
                let y = self.grid.align_i32(new_pos.y);
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
                rect.apply_to_trigger(trigger);
                true
            } else {
                false
            }
    }

    fn drag_start(&mut self, item_pos: Pos2, mouse_pos: Pos2) {
        self.dragging_item = true;
        self.drag_item_origin = item_pos;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = None;
    }

    fn drag_move(&mut self, mouse_pos: Pos2, room: &mut Room) -> bool {
        let new_pos = self.drag_item_origin + (mouse_pos - self.drag_mouse_origin);
        Self::move_item(&mut self.selected_item, new_pos, room, &mut self.screen_pos, self.lock_maps, &self.grid).unwrap_or(false)
    }

    fn drag_stop(&mut self) {
        self.dragging_item = false;
        self.resize_border = None;
    }

    fn handle_mouse_hover(&mut self, resp: &egui::Response, mouse_pos: Pos2, room: &mut Room, assets: &RoomEditorAssetLists) {
        let keys_pressed = resp.ctx.input(|i| i.modifiers);
        if keys_pressed.alt {
            if resp.dragged() {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                resp.ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
        } else if keys_pressed.ctrl {
            resp.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
        } else if let Some(border) = Self::get_trigger_border(self.selected_item, room, mouse_pos, self.zoom, assets) {
            resp.ctx.set_cursor_icon(border.cursor());
        }
    }

    fn handle_mouse_down(&mut self, resp: &egui::Response, mouse_pos: Pos2, room: &mut Room, assets: &RoomEditorAssetLists) {
        if resp.drag_stopped() {
            self.drag_stop();
            return;
        }

        if self.dragging_item && self.selected_item.is_some() {
            if ! resp.dragged_by(egui::PointerButton::Primary) {
                self.drag_stop();
                return;
            }
            if let Some(border) = self.resize_border {
                resp.ctx.set_cursor_icon(border.cursor());
                if self.resize_move(mouse_pos, room, border, assets) {
                    return;
                }
            }
            if self.drag_move(mouse_pos, room) {
                return;
            }
            self.drag_stop();
        }

        // drag trigger border
        if resp.drag_started() &&
            resp.dragged_by(egui::PointerButton::Primary) &&
            self.selected_item.is_trigger() &&
            let Some(border) = Self::get_trigger_border(self.selected_item, room, mouse_pos, self.zoom, assets) &&
            let Some(rect) = Self::get_item_rect(self.selected_item, room, assets, &self.screen_pos) {
                self.resize_start(border, rect, mouse_pos);
                return;
            }

        // click/drag selected trigger
        if self.selected_item.is_trigger() {
            let rect = Self::get_item_rect(self.selected_item, room, assets, &self.screen_pos).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag selected screen
        if self.show_screen && self.selected_item.is_screen() {
            let rect = self.screen_pos.get_rect();
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag trigger under the cursor
        for index in 0..room.triggers.len() {
            let item = RoomItemRef::Trigger(index);
            let rect = Self::get_item_rect(item, room, assets, &self.screen_pos).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.set_selected_item(item);
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag screen
        if self.show_screen {
            let rect = self.screen_pos.get_rect();
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.set_selected_item(RoomItemRef::Screen);
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag selected map
        if self.selected_item.is_map() {
            let rect = Self::get_item_rect(self.selected_item, room, assets, &self.screen_pos).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // click/drag map under the cursor
        for index in 0..room.maps.len() {
            let item = RoomItemRef::Map(index);
            let rect = Self::get_item_rect(item, room, assets, &self.screen_pos).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.set_selected_item(item);
                if resp.drag_started() {
                    self.drag_start(rect.min, mouse_pos);
                }
                return;
            }
        }

        // left-click nowhere deselects selected item
        if resp.dragged_by(egui::PointerButton::Primary) {
            self.set_selected_item(RoomItemRef::None);
        }
    }

    pub fn set_zoom<F>(&mut self, zoom: f32, center_delta: Vec2, canvas_size: Vec2, get_to_canvas: &F, room_rect: Rect)
    where F: Fn(f32, Vec2) -> RectTransform {
        let zoom = zoom.max(0.25);
        let zoom_delta = zoom / self.zoom;
        self.zoom = zoom;
        self.scroll = center_delta - (center_delta - self.scroll) * zoom_delta;

        let to_canvas = get_to_canvas(self.zoom, self.scroll);
        let trans_room_size = to_canvas.transform_rect(room_rect).size();
        self.clip_scroll(canvas_size, trans_room_size);
    }

    pub fn clip_scroll(&mut self, canvas_size: Vec2, trans_room_size: Vec2) {
        self.scroll = self.scroll.max(canvas_size - (trans_room_size + 2.0 * BORDER_SIZE)).min(Vec2::ZERO);
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

    fn draw_map(ui: &mut egui::Ui, wc: &mut WindowContext, to_canvas: &RectTransform, map_pos: Pos2, map_data: &MapData, tileset: &Tileset) {
        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_items() { continue; }
                let draw_rect = to_canvas.transform_rect(Self::get_tile_rect(x, y, map_pos));
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, draw_rect);
            }
        }

        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_items() { continue; }
                let draw_rect = to_canvas.transform_rect(Self::get_tile_rect(x, y, map_pos));
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, draw_rect);
            }
        }
    }

    fn draw_trigger_sprite(ui: &mut egui::Ui, wc: &mut WindowContext, to_canvas: &RectTransform,
                           rect: Rect, sprite: &Sprite, frame: u32) {
        let draw_rect = to_canvas.transform_rect(rect);
        let texture = sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        Image::from_texture((texture.id(), sprite.get_item_size())).uv(sprite.get_item_uv(frame)).paint_at(ui, draw_rect);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, room: &mut Room, assets: &RoomEditorAssetLists) {
        self.grid.align = ui.ctx().input(|i| i.modifiers.shift);

        let min_size = ui.available_size();
        let (response, mut painter) = ui.allocate_painter(min_size, Sense::drag());
        let response_rect = response.rect;

        let room_size = Self::get_room_size(room, assets.maps);
        let room_rect = Rect::from_min_size(Pos2::ZERO, room_size);
        let canvas_rect = response_rect.expand2(-Vec2::splat(1.0));
        let to_canvas_from_zoom = move |zoom, scroll| {
            RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, room_size),
                Rect::from_min_size(canvas_rect.min + BORDER_SIZE + scroll, room_size * zoom),
            )
        };

        let to_canvas = to_canvas_from_zoom(self.zoom, self.scroll);
        let bg_rect = Rect {
            min: response_rect.min,
            max: Pos2 {
                x: response_rect.max.x.min(response_rect.min.x + room_size.x * self.zoom + 2.0 + 2.0*BORDER_SIZE.x),
                y: response_rect.max.y.min(response_rect.min.y + room_size.y * self.zoom + 2.0 + 2.0*BORDER_SIZE.y),
            },
        };
        painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0, 0));
        let stroke = egui::Stroke::new(1.0, Color32::WHITE);
        painter.rect_stroke(bg_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Middle);
        painter.shrink_clip_rect(canvas_rect);
        ui.shrink_clip_rect(canvas_rect);
        Self::draw_outline_rect(&painter, canvas_rect);

        if canvas_rect.width() == 0.0 || canvas_rect.height() == 0.0 || room_rect.width() == 0.0 || room_rect.height() == 0.0 {
            return; // nothing to do!
        }

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.size(), to_canvas.transform_rect(room_rect).size());

        // draw maps
        for room_map in room.maps.iter() {
            if let Some(map_data) = assets.maps.get(&room_map.map_id) && let Some(tileset) = assets.tilesets.get(&map_data.tileset_id) {
                let map_rect = Self::get_map_rect(room_map, map_data);
                Self::draw_map(ui, wc, &to_canvas, map_rect.min, map_data, tileset);
            }
        }

        // draw triggers
        for trigger in room.triggers.iter() {
            let rect = Self::get_trigger_rect(trigger, assets);
            match trigger.trigger_type {
                RoomTriggerType::Unknown {..} |
                RoomTriggerType::Door {..} |
                RoomTriggerType::PlayerSpawn {..} |
                RoomTriggerType::Trap {..} => {
                    Self::draw_outline_rect(&painter, to_canvas.transform_rect(rect));
                }
                RoomTriggerType::EnemySpawn { animation_id } => {
                    if let Some(animation) = assets.animations.get(&animation_id) &&
                        let Some(sprite) = assets.sprites.get(&animation.sprite_id) {
                            let sprite_frame = animation.loops.first()
                                .and_then(|aloop| aloop.frame_indices.first())
                                .and_then(|frame| frame.head_index)
                                .unwrap_or(0);
                            Self::draw_trigger_sprite(ui, wc, &to_canvas, rect, sprite, sprite_frame as u32);
                            Self::draw_outline_rect(&painter, to_canvas.transform_rect(rect));
                        }
                }
            }
        }

        // draw screen rect
        if self.show_screen {
            let rect = self.screen_pos.get_rect();
            if self.selected_item.is_screen() {
                Self::draw_selection_rect(&painter, to_canvas.transform_rect(rect));
            } else {
                Self::draw_outline_rect(&painter, to_canvas.transform_rect(rect));
            }
        }

        // outline selected map
        if let RoomItemRef::Map(map_index) = self.selected_item &&
            let Some(room_map) = room.maps.get(map_index) &&
            let Some(map_data) = assets.maps.get(&room_map.map_id) {
                let rect = Self::get_map_rect(room_map, map_data);
                Self::draw_selection_rect(&painter, to_canvas.transform_rect(rect));
            }

        // outline selected trigger
        if let RoomItemRef::Trigger(trg_index) = self.selected_item &&
            let Some(trigger) = room.triggers.get(trg_index) {
                let rect = Self::get_trigger_rect(trigger, assets);
                Self::draw_selection_rect(&painter, to_canvas.transform_rect(rect));
            }

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // check hover
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let mouse_pos = to_canvas.inverse() * hover_pos;
            self.handle_mouse_hover(&response, mouse_pos, room, assets);
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || keys_pressed.alt {
            self.scroll += response.drag_delta();
            self.clip_scroll(canvas_rect.size(), to_canvas.transform_rect(room_rect).size());
        }

        // check click
        if let Some(pointer_pos) = response.interact_pointer_pos() && ! (keys_pressed.alt || keys_pressed.ctrl) {
            let click_pos = to_canvas.inverse() * pointer_pos;
            self.handle_mouse_down(&response, click_pos, room, assets);
        }

        // check zoom (must be last)
        if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
            let zoom_delta = if keys_pressed.ctrl && response.dragged_by(egui::PointerButton::Primary) {
                (response.drag_delta().y * -0.01).exp()
            } else {
                ui.input(|i| i.zoom_delta())
            };
            if zoom_delta != 1.0 {
                self.set_zoom(self.zoom * zoom_delta, hover_pos - canvas_rect.min, canvas_rect.size(), &to_canvas_from_zoom, room_rect);
            }
        }
    }
}
