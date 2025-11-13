use egui::{Vec2, Sense, Rect, Pos2, Image, Color32};
use egui::emath::RectTransform;

use crate::data_asset::{Room, RoomMap, RoomEntity, RoomTrigger, MapData, Tileset, Sprite, AssetList};
use crate::misc::{ImageCollection, WindowContext};

use super::{MapLayer, RectBorder, TILE_SIZE, get_map_layer_tile};
use super::super::room::{RoomEditorAssetLists, RoomItemRef};

const BORDER_SIZE: Vec2 = Vec2::splat(5.0);
const DRAG_BORDER_FUDGE_SIZE: f32 = 8.0;

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

fn get_entity_rect(entity: &RoomEntity, sprite: &Sprite) -> Rect {
    let ent_pos = Pos2::new(entity.x as f32, entity.y as f32);
    let ent_size = Vec2::new(sprite.width as f32, sprite.height as f32);
    egui::Rect {
        min: ent_pos,
        max: ent_pos + ent_size,
    }
}

fn get_trigger_rect(trigger: &RoomTrigger) -> Rect {
    let trg_pos = Pos2::new(trigger.x as f32, trigger.y as f32);
    let trg_size = Vec2::new(trigger.width as f32, trigger.height as f32);
    egui::Rect {
        min: trg_pos,
        max: trg_pos + trg_size,
    }
}

fn get_item_rect(item: RoomItemRef, room: &Room, assets: &RoomEditorAssetLists) -> Option<Rect> {
    match item {
        RoomItemRef::None => None,

        RoomItemRef::Map(map_index) => {
            let room_map = room.maps.get(map_index)?;
            let map_data = assets.maps.get(&room_map.map_id)?;
            Some(get_map_rect(room_map, map_data))
        },

        RoomItemRef::Entity(ent_index) => {
            let entity = room.entities.get(ent_index)?;
            let animation = assets.animations.get(&entity.animation_id)?;
            let sprite = assets.sprites.get(&animation.sprite_id)?;
            Some(get_entity_rect(entity, sprite))
        },

        RoomItemRef::Trigger(trg_index) => {
            let trigger = room.triggers.get(trg_index)?;
            Some(get_trigger_rect(trigger))
        },
    }
}

fn move_item(item: RoomItemRef, pos: Pos2, room: &mut Room) -> Option<bool> {
    match item {
        RoomItemRef::None => None,

        RoomItemRef::Map(map_index) => {
            let room_map = room.maps.get_mut(map_index)?;
            room_map.x = (pos.x / TILE_SIZE).round().clamp(0.0, 1024.0) as u16;
            room_map.y = (pos.y / TILE_SIZE).round().clamp(0.0, 1024.0) as u16;
            Some(true)
        },

        RoomItemRef::Entity(ent_index) => {
            let entity = room.entities.get_mut(ent_index)?;
            entity.x = pos.x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            entity.y = pos.y.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            Some(true)
        },

        RoomItemRef::Trigger(trg_index) => {
            let trigger = room.triggers.get_mut(trg_index)?;
            trigger.x = pos.x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            trigger.y = pos.y.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            Some(true)
        },
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

fn get_trigger_border(item: RoomItemRef, room: &mut Room, pos: Pos2, zoom: f32) -> Option<RectBorder> {
    if let RoomItemRef::Trigger(trg_index) = item {
        let trigger = room.triggers.get(trg_index)?;
        let rect = get_trigger_rect(trigger);
        get_rect_border(rect, pos, zoom)
    } else {
        None
    }
}

pub struct RoomEditorState {
    pub zoom: f32,
    pub scroll: Vec2,
    pub selected_item: RoomItemRef,
    resize_border: Option<RectBorder>,
    drag_item: RoomItemRef,
    drag_item_origin: Pos2,
    drag_mouse_origin: Pos2,
}

impl RoomEditorState {
    pub fn new() -> Self {
        RoomEditorState {
            zoom: 0.5,
            scroll: Vec2::ZERO,
            selected_item: RoomItemRef::None,
            resize_border: None,
            drag_item: RoomItemRef::None,
            drag_item_origin: Pos2::ZERO,
            drag_mouse_origin: Pos2::ZERO,
        }
    }

    fn resize_start(&mut self, item: RoomItemRef, border: RectBorder, item_rect: Rect, mouse_pos: Pos2) {
        self.drag_item = item;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = Some(border);
        self.drag_item_origin = match border {
            RectBorder::Top | RectBorder::Left | RectBorder::TopLeft => item_rect.min,
            RectBorder::Bottom | RectBorder::Right | RectBorder::BottomRight => item_rect.max,
            RectBorder::BottomLeft => Pos2::new(item_rect.min.x, item_rect.max.y),
            RectBorder::TopRight => Pos2::new(item_rect.max.x, item_rect.min.y),
        };
    }

    fn resize_move(&mut self, mouse_pos: Pos2, room: &mut Room, border: RectBorder) -> bool {
        let new_pos = self.drag_item_origin + (mouse_pos - self.drag_mouse_origin);
        if let RoomItemRef::Trigger(index) = self.drag_item && let Some(trigger) = room.triggers.get_mut(index) {
            let trigger_pos = Pos2::new(trigger.x as f32, trigger.y as f32);
            match border {
                RectBorder::Bottom => { trigger.height = (new_pos - trigger_pos).y.clamp(0.0, i16::MAX as f32) as i16; }
                RectBorder::Right => { trigger.width = (new_pos - trigger_pos).x.clamp(0.0, i16::MAX as f32) as i16; }
                RectBorder::BottomRight => {
                    let new_size = new_pos - trigger_pos;
                    trigger.width = new_size.x.clamp(0.0, i16::MAX as f32) as i16;
                    trigger.height = new_size.y.clamp(0.0, i16::MAX as f32) as i16;
                }
                RectBorder::Top => {
                    let old_y = trigger.y;
                    trigger.y = new_pos.y.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.height = (trigger.height + old_y - trigger.y).max(0);
                }
                RectBorder::Left => {
                    let old_x = trigger.x;
                    trigger.x = new_pos.x.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.width = (trigger.width + old_x - trigger.x).max(0);
                }
                RectBorder::TopLeft => {
                    let (old_x, old_y) = (trigger.x, trigger.y);
                    trigger.x = new_pos.x.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.y = new_pos.y.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.width = (trigger.width + old_x - trigger.x).max(0);
                    trigger.height = (trigger.height + old_y - trigger.y).max(0);
                }
                RectBorder::TopRight => {
                    let old_y = trigger.y;
                    trigger.y = new_pos.y.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.height = (trigger.height + old_y - trigger.y).max(0);
                    trigger.width = (new_pos - trigger_pos).x.clamp(0.0, i16::MAX as f32) as i16;
                }
                RectBorder::BottomLeft => {
                    trigger.height = (new_pos - trigger_pos).y.clamp(0.0, i16::MAX as f32) as i16;
                    let old_x = trigger.x;
                    trigger.x = new_pos.x.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                    trigger.width = (trigger.width + old_x - trigger.x).max(0);
                }
            }
            true
        } else {
            false
        }
    }

    fn drag_start(&mut self, item: RoomItemRef, item_pos: Pos2, mouse_pos: Pos2) {
        self.drag_item = item;
        self.drag_item_origin = item_pos;
        self.drag_mouse_origin = mouse_pos;
        self.resize_border = None;
    }

    fn drag_move(&mut self, mouse_pos: Pos2, room: &mut Room) -> bool {
        let new_pos = self.drag_item_origin + (mouse_pos - self.drag_mouse_origin);
        move_item(self.drag_item, new_pos, room).unwrap_or(false)
    }

    fn drag_stop(&mut self) {
        self.drag_item = RoomItemRef::None;
        self.resize_border = None;
    }

    fn handle_mouse_hover(&mut self, resp: &egui::Response, mouse_pos: Pos2, room: &mut Room, _assets: &RoomEditorAssetLists) -> Option<()> {
        if let Some(border) = get_trigger_border(self.selected_item, room, mouse_pos, self.zoom) {
            resp.ctx.set_cursor_icon(border.cursor());
        }
        None
    }

    fn handle_mouse_down(&mut self, resp: &egui::Response, mouse_pos: Pos2, room: &mut Room, assets: &RoomEditorAssetLists) -> Option<()> {
        if resp.drag_stopped() {
            self.drag_stop();
            return None;
        }

        if self.drag_item.is_some() {
            if ! resp.dragged_by(egui::PointerButton::Primary) {
                self.drag_stop();
                return None;
            }
            if let Some(border) = self.resize_border {
                resp.ctx.set_cursor_icon(border.cursor());
                if  self.resize_move(mouse_pos, room, border) {
                    return None;
                }
            }
            if self.drag_move(mouse_pos, room) {
                return None;
            }
            self.drag_stop();
        }

        // drag trigger border
        if resp.drag_started() &&
            resp.dragged_by(egui::PointerButton::Primary) &&
            self.selected_item.is_trigger() &&
            let Some(border) = get_trigger_border(self.selected_item, room, mouse_pos, self.zoom) &&
            let Some(rect) = get_item_rect(self.selected_item, room, assets) {
                self.resize_start(self.selected_item, border, rect, mouse_pos);
                return None;
            }

        // click/drag selected trigger/entity
        if self.selected_item.is_trigger() || self.selected_item.is_entity() {
            let rect = get_item_rect(self.selected_item, room, assets).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                if resp.drag_started() {
                    self.drag_start(self.selected_item, rect.min, mouse_pos);
                }
                return None;
            }
        }

        // click/drag entity under the cursor
        for index in 0..room.entities.len() {
            let item = RoomItemRef::Entity(index);
            let rect = get_item_rect(item, room, assets).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.selected_item = item;
                if resp.drag_started() {
                    self.drag_start(self.selected_item, rect.min, mouse_pos);
                }
                return None;
            }
        }

        // click/drag trigger under the cursor
        for index in 0..room.triggers.len() {
            let item = RoomItemRef::Trigger(index);
            let rect = get_item_rect(item, room, assets).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.selected_item = item;
                if resp.drag_started() {
                    self.drag_start(self.selected_item, rect.min, mouse_pos);
                }
                return None;
            }
        }

        // click/drag map under the cursor
        for index in 0..room.maps.len() {
            let item = RoomItemRef::Map(index);
            let rect = get_item_rect(item, room, assets).unwrap_or(Rect::NOTHING);
            if rect.contains(mouse_pos) && resp.dragged_by(egui::PointerButton::Primary) {
                self.selected_item = item;
                if resp.drag_started() {
                    self.drag_start(self.selected_item, rect.min, mouse_pos);
                }
                return None;
            }
        }

        self.selected_item = RoomItemRef::None;
        None
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
    let (image, texture) = ImageCollection::load_asset(tileset, wc.tex_man, wc.egui.ctx, false);
    for y in 0..map_data.bg_height {
        for x in 0..map_data.bg_width {
            let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let draw_rect = to_canvas.transform_rect(get_tile_rect(x, y, map_pos));
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, draw_rect);
        }
    }

    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let draw_rect = to_canvas.transform_rect(get_tile_rect(x, y, map_pos));
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, draw_rect);
        }
    }
}

fn draw_entity(ui: &mut egui::Ui, wc: &mut WindowContext, to_canvas: &RectTransform, entity_rect: Rect, sprite: &Sprite, frame: u32) {
    let draw_rect = to_canvas.transform_rect(entity_rect);
    let (image, texture) = ImageCollection::load_asset(sprite, wc.tex_man, wc.egui.ctx, false);
    Image::from_texture((texture.id(), image.get_item_size())).uv(image.get_item_uv(frame)).paint_at(ui, draw_rect);
}

pub fn room_editor(ui: &mut egui::Ui, wc: &mut WindowContext, state: &mut RoomEditorState, room: &mut Room, assets: &RoomEditorAssetLists) {
    let min_size = ui.available_size();
    let (response, mut painter) = ui.allocate_painter(min_size, Sense::drag());
    let response_rect = response.rect;

    let room_size = get_room_size(room, assets.maps);
    let room_rect = Rect::from_min_size(Pos2::ZERO, room_size);
    let canvas_rect = response_rect.expand2(-Vec2::splat(1.0));
    let to_canvas_from_zoom = move |zoom, scroll| {
        RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, room_size),
            Rect::from_min_size(canvas_rect.min + BORDER_SIZE + scroll, room_size * zoom),
        )
    };

    let to_canvas = to_canvas_from_zoom(state.zoom, state.scroll);
    let bg_rect = Rect {
        min: response_rect.min,
        max: Pos2 {
            x: response_rect.max.x.min(response_rect.min.x + room_size.x + 2.0),
            y: response_rect.max.y.min(response_rect.min.y + room_size.y + 2.0),
        },
    };
    painter.rect_filled(bg_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0, 0));
    let stroke = egui::Stroke::new(1.0, Color32::WHITE);
    painter.rect_stroke(bg_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Middle);
    painter.shrink_clip_rect(canvas_rect);
    ui.shrink_clip_rect(canvas_rect);
    draw_outline_rect(&painter, canvas_rect);

    // limit scroll in case we've been resized
    state.clip_scroll(canvas_rect.size(), to_canvas.transform_rect(room_rect).size());

    // draw maps
    for room_map in room.maps.iter() {
        if let Some(map_data) = assets.maps.get(&room_map.map_id) && let Some(tileset) = assets.tilesets.get(&map_data.tileset_id) {
            let map_rect = get_map_rect(room_map, map_data);
            draw_map(ui, wc, &to_canvas, map_rect.min, map_data, tileset);
        }
    }

    // draw triggers
    for trigger in room.triggers.iter() {
        let trg_rect = get_trigger_rect(trigger);
        draw_outline_rect(&painter, to_canvas.transform_rect(trg_rect));
    }

    // draw entities
    for entity in room.entities.iter() {
        if let Some(animation) = assets.animations.get(&entity.animation_id) && let Some(sprite) = assets.sprites.get(&animation.sprite_id) {
            let sprite_frame = animation.loops.first()
                .and_then(|aloop| aloop.frame_indices.first())
                .and_then(|frame| frame.head_index)
                .unwrap_or(0);
            let ent_rect = get_entity_rect(entity, sprite);
            draw_entity(ui, wc, &to_canvas, ent_rect, sprite, sprite_frame as u32);
            draw_outline_rect(&painter, to_canvas.transform_rect(ent_rect));
        }
    }

    // outline selected map
    if let RoomItemRef::Map(map_index) = state.selected_item &&
        let Some(room_map) = room.maps.get(map_index) &&
        let Some(map_data) = assets.maps.get(&room_map.map_id) {
            let rect = get_map_rect(room_map, map_data);
            draw_selection_rect(&painter, to_canvas.transform_rect(rect));
        }

    // outline selected entity
    if let RoomItemRef::Entity(ent_index) = state.selected_item &&
        let Some(entity) = room.entities.get(ent_index) &&
        let Some(sprite) = assets.animations.get(&entity.animation_id).and_then(|anim| assets.sprites.get(&anim.sprite_id)) {
            let rect = get_entity_rect(entity, sprite);
            draw_selection_rect(&painter, to_canvas.transform_rect(rect));
        }

    // outline selected trigger
    if let RoomItemRef::Trigger(trg_index) = state.selected_item &&
        let Some(trigger) = room.triggers.get(trg_index) {
            let rect = get_trigger_rect(trigger);
            draw_selection_rect(&painter, to_canvas.transform_rect(rect));
        }

    // check hover
    if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
        let mouse_pos = to_canvas.inverse() * hover_pos;
        state.handle_mouse_hover(&response, mouse_pos, room, assets);
    }

    // check pan
    if response.dragged_by(egui::PointerButton::Middle) {
        state.scroll += response.drag_delta();
        state.clip_scroll(canvas_rect.size(), to_canvas.transform_rect(room_rect).size());
    }

    // check click
    if let Some(pointer_pos) = response.interact_pointer_pos() {
        let click_pos = to_canvas.inverse() * pointer_pos;
        state.handle_mouse_down(&response, click_pos, room, assets);
    }

    // check zoom (must be last)
    if response.contains_pointer() && let Some(hover_pos) = response.hover_pos() {
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            state.set_zoom(state.zoom * zoom_delta, hover_pos - canvas_rect.min, canvas_rect.size(), &to_canvas_from_zoom, room_rect);
        }
    }
}
