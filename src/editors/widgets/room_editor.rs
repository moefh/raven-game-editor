use egui::{Vec2, Sense, Rect, Pos2, Image, Color32};

use crate::data_asset::{/*DataAssetId, AssetList,*/ Room, RoomMap, RoomEntity, RoomTrigger, MapData, Tileset, Sprite/*, SpriteAnimation*/};
use crate::misc::{ImageCollection, WindowContext};

use super::{MapLayer, TILE_SIZE, get_map_layer_tile};
use super::super::room::{RoomEditorAssetLists, RoomItemRef};

pub struct RoomEditorState {
    pub zoom: f32,
    pub scroll: Vec2,
    pub selected_item: RoomItemRef,
}

impl RoomEditorState {
    pub fn new() -> Self {
        RoomEditorState {
            zoom: 0.5,
            scroll: Vec2::ZERO,
            selected_item: RoomItemRef::None,
        }
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

fn get_map_rect(room_map: &RoomMap, map_data: &MapData, canvas_rect: Rect, state: &RoomEditorState) -> Rect {
    let map_delta = state.zoom * Vec2::new(room_map.x as f32 * TILE_SIZE, room_map.y as f32 * TILE_SIZE);
    let map_size = state.zoom * Vec2::new(map_data.width as f32 * TILE_SIZE, map_data.height as f32 * TILE_SIZE);
    egui::Rect {
        min: canvas_rect.min + Vec2::splat(5.0) + state.scroll + map_delta,
        max: canvas_rect.min + Vec2::splat(5.0) + state.scroll + map_delta + map_size,
    }
}

fn get_entity_rect(entity: &RoomEntity, sprite: &Sprite, canvas_rect: Rect, state: &RoomEditorState) -> Rect {
    let ent_delta = state.zoom * Vec2::new(entity.x as f32, entity.y as f32);
    let ent_size = state.zoom * Vec2::new(sprite.width as f32, sprite.height as f32);
    egui::Rect {
        min: canvas_rect.min + Vec2::splat(5.0) + state.scroll + ent_delta,
        max: canvas_rect.min + Vec2::splat(5.0) + state.scroll + ent_delta + ent_size,
    }
}

fn get_trigger_rect(trigger: &RoomTrigger, canvas_rect: Rect, state: &RoomEditorState) -> Rect {
    let trg_delta = state.zoom * Vec2::new(trigger.x as f32, trigger.y as f32);
    let trg_size = state.zoom * Vec2::new(trigger.width as f32, trigger.height as f32);
    egui::Rect {
        min: canvas_rect.min + Vec2::splat(5.0) + state.scroll + trg_delta,
        max: canvas_rect.min + Vec2::splat(5.0) + state.scroll + trg_delta + trg_size,
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

fn draw_map(ui: &mut egui::Ui, wc: &mut WindowContext, map_pos: Pos2, map_data: &MapData, tileset: &Tileset, state: &RoomEditorState) {
    let (image, texture) = ImageCollection::load_asset(tileset, wc.tex_man, wc.egui.ctx, false);
    for y in 0..map_data.bg_height {
        for x in 0..map_data.bg_width {
            let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let tile_rect = get_tile_rect(x, y, state.zoom, map_pos);
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
        }
    }

    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
            if tile == 0xff || tile >= image.num_items { continue; }
            let tile_rect = get_tile_rect(x, y, state.zoom, map_pos);
            Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(image.get_item_uv(tile)).paint_at(ui, tile_rect);
        }
    }
}

fn draw_entity(ui: &mut egui::Ui, wc: &mut WindowContext, entity_rect: Rect, sprite: &Sprite, frame: u32) {
    let (image, texture) = ImageCollection::load_asset(sprite, wc.tex_man, wc.egui.ctx, false);
    Image::from_texture((texture.id(), image.get_item_size())).uv(image.get_item_uv(frame)).paint_at(ui, entity_rect);
}

pub fn room_editor(ui: &mut egui::Ui, wc: &mut WindowContext, state: &mut RoomEditorState, room: &mut Room, assets: &RoomEditorAssetLists) {
    let min_size = ui.available_size();
    let (response, painter) = ui.allocate_painter(min_size, Sense::drag());
    let response_rect = response.rect;

    let canvas_rect = response_rect;
    ui.shrink_clip_rect(canvas_rect);
    painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0, 0));

    // draw maps
    for room_map in room.maps.iter() {
        if let Some(map_data) = assets.maps.get(&room_map.map_id) && let Some(tileset) = assets.tilesets.get(&map_data.tileset_id) {
            let map_rect = get_map_rect(room_map, map_data, canvas_rect, state);
            draw_map(ui, wc, map_rect.min, map_data, tileset, state);
        }
    }

    // draw triggers
    for trigger in room.triggers.iter() {
        let trg_rect = get_trigger_rect(trigger, canvas_rect, state);
        draw_outline_rect(&painter, trg_rect);
    }

    // draw entities
    for entity in room.entities.iter() {
        if let Some(animation) = assets.animations.get(&entity.animation_id) && let Some(sprite) = assets.sprites.get(&animation.sprite_id) {
            let sprite_frame = animation.loops.first()
                .and_then(|aloop| aloop.frame_indices.first())
                .and_then(|frame| frame.head_index)
                .unwrap_or(0);
            let ent_rect = get_entity_rect(entity, sprite, canvas_rect, state);
            draw_entity(ui, wc, ent_rect, sprite, sprite_frame as u32);
            draw_outline_rect(&painter, ent_rect);
        }
    }

    // outline selected map
    if let RoomItemRef::Map(map_index) = state.selected_item &&
        let Some(room_map) = room.maps.get(map_index) &&
        let Some(map_data) = assets.maps.get(&room_map.map_id) {
            let rect = get_map_rect(room_map, map_data, canvas_rect, state);
            draw_selection_rect(&painter, rect);
        }

    // outline selected entity
    if let RoomItemRef::Entity(ent_index) = state.selected_item &&
        let Some(entity) = room.entities.get(ent_index) &&
        let Some(sprite) = assets.animations.get(&entity.animation_id).and_then(|anim| assets.sprites.get(&anim.sprite_id)) {
            let rect = get_entity_rect(entity, sprite, canvas_rect, state);
            draw_selection_rect(&painter, rect);
        }

    // outline selected trigger
    if let RoomItemRef::Trigger(trg_index) = state.selected_item &&
        let Some(trigger) = room.triggers.get(trg_index) {
            let rect = get_trigger_rect(trigger, canvas_rect, state);
            draw_selection_rect(&painter, rect);
        }
}
