use egui::{
    Vec2,
    Rect,
    Pos2,
    Image,
};
use egui::emath::RectTransform;

use crate::image::{
    TextureSlot,
    ImageCollection,
};
use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    SpriteAnimation,
    TileAnimation,
    Tileset,
    MapData,
    Sprite,
    Room,
    RoomMap,
};

use super::super::{
    WindowContext,
};
use super::super::editors::{
    get_animated_tile,
    get_animation_step,
    get_map_layer_tile,
    MapLayer,
    RoomSize,
};

pub const TILE_SIZE: f32 = crate::data_asset::Tileset::TILE_SIZE as f32;

pub struct GameRunnerWidget {
    parent_window_id: egui::Id,
    pub zoom: f32,
    pub scroll: Vec2,
    pub room_id: Option<DataAssetId>,
    pub player_anim_id: Option<DataAssetId>,
}

impl GameRunnerWidget {
    pub const WIDTH: f32 = 320.0;
    pub const HEIGHT: f32 = 240.0;
    pub const SCREEN_SIZE: Vec2 = Vec2 { x: Self::WIDTH, y: Self::HEIGHT };
    const PLAYER_ANIMATION: &str = "bunny";

    pub fn new(parent_window_id: egui::Id) -> Self {
        GameRunnerWidget {
            parent_window_id,
            zoom: 2.0,
            scroll: Vec2::ZERO,
            room_id: None,
            player_anim_id: None,
        }
    }

    fn get_sprite_animation_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a SpriteAnimation> {
        store.assets.animations.iter().find(|asset| asset.asset.name == name)
    }

    pub fn reset(&mut self, room_id: DataAssetId, store: &DataAssetStore) -> bool {
        self.room_id = Some(room_id);
        if let Some(anim) = Self::get_sprite_animation_by_name(store, Self::PLAYER_ANIMATION) {
            self.player_anim_id = Some(anim.asset.id);
            true
        } else {
            self.player_anim_id = None;
            false
        }
    }

    fn get_room_size(size: RoomSize) -> Vec2 {
        Vec2 {
            x: size.width as f32 * TILE_SIZE,
            y: size.height as f32 * TILE_SIZE,
        }
    }

    fn get_map_rect(room_map: &RoomMap, map_data: &MapData) -> Rect {
        let map_pos = Pos2::new(room_map.x as f32, room_map.y as f32);
        let map_size = Vec2::new(map_data.width as f32, map_data.height as f32);
        TILE_SIZE * egui::Rect::from_min_size(map_pos, map_size)
    }

    fn get_tile_rect(x: u32, y: u32, map_pos: Pos2) -> Rect {
        let tile_pos = TILE_SIZE * Vec2::new(x as f32, y as f32);
        Rect::from_min_size(map_pos + tile_pos, Vec2::splat(TILE_SIZE))
    }

    fn draw_map_para_layer(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        to_canvas: &RectTransform,
        map_pos: Pos2,
        map_data: &MapData,
        tileset: &Tileset
    ) {
        for y in 0..map_data.para_height {
            for x in 0..map_data.para_width {
                let tile = get_map_layer_tile(map_data, MapLayer::Parallax, x, y);
                if tile == MapData::NO_TILE || tile as u32 >= tileset.num_tiles { continue; }
                let draw_rect = to_canvas.transform_rect(Self::get_tile_rect(x, y, map_pos));
                let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(tileset.get_item_uv(tile as u32)).paint_at(ui, draw_rect);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_map_bg_layer(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        to_canvas: &RectTransform,
        map_pos: Pos2,
        map_data: &MapData,
        tileset: &Tileset,
        tile_anim: Option<&TileAnimation>,
        anim_tileset: Option<&Tileset>,
        animation_step: u32,
    ) {
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Background, x, y);
                if tile == MapData::NO_TILE { continue; }
                let (tile, use_tileset) = if let Some(new_tile) = get_animated_tile(
                    tile,
                    MapLayer::Background,
                    get_map_layer_tile(map_data, MapLayer::Animation, x, y),
                    tile_anim,
                    animation_step
                ) {
                    if let Some(anim_tileset) = anim_tileset {
                        (new_tile, anim_tileset)
                    } else {
                        (new_tile, tileset)
                    }
                } else {
                    (tile, tileset)
                };
                if tile as u32 >= use_tileset.num_tiles { continue; }
                let draw_rect = to_canvas.transform_rect(Self::get_tile_rect(x, y, map_pos));
                let slot = if map_data.para_width == 0 || map_data.para_height == 0 {
                    TextureSlot::Opaque
                } else {
                    TextureSlot::Transparent
                };
                let texture = use_tileset.texture(wc.tex_man, wc.egui.ctx, slot);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(use_tileset.get_item_uv(tile as u32)).paint_at(ui, draw_rect);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_map_fg_layer(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        to_canvas: &RectTransform,
        map_pos: Pos2,
        map_data: &MapData,
        tileset: &Tileset,
        tile_anim: Option<&TileAnimation>,
        anim_tileset: Option<&Tileset>,
        animation_step: u32,
    ) {
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                if tile == MapData::NO_TILE { continue; }
                let (tile, use_tileset) = if let Some(new_tile) = get_animated_tile(
                    tile,
                    MapLayer::Background,
                    get_map_layer_tile(map_data, MapLayer::Animation, x, y),
                    tile_anim,
                    animation_step
                ) {
                    if let Some(anim_tileset) = anim_tileset {
                        (new_tile, anim_tileset)
                    } else {
                        (new_tile, tileset)
                    }
                } else {
                    (tile, tileset)
                };
                if tile as u32 >= use_tileset.num_tiles { continue; }
                let draw_rect = to_canvas.transform_rect(Self::get_tile_rect(x, y, map_pos));
                let texture = use_tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                Image::from_texture((texture.id(), Vec2::splat(TILE_SIZE))).uv(use_tileset.get_item_uv(tile as u32)).paint_at(ui, draw_rect);
            }
        }
    }

    fn draw_frame(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        _player_sprite: &Sprite,
        _player_anim: &SpriteAnimation,
        store: &DataAssetStore
    ) {
        let min_size = self.zoom * Self::SCREEN_SIZE.min(ui.available_size());
        let response = ui.allocate_response(min_size, egui::Sense::click());
        let extra = response.rect.size() - min_size;
        let canvas_rect = Rect::from_min_size(response.rect.min + 0.5 * extra, response.rect.size());
        ui.shrink_clip_rect(canvas_rect);

        let room_size_in_tiles = RoomSize::from_room(room, &store.assets.maps);
        let room_size = Self::get_room_size(room_size_in_tiles);
        let to_canvas = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Self::SCREEN_SIZE),
            Rect::from_min_size(canvas_rect.min + self.zoom * self.scroll, Self::SCREEN_SIZE * self.zoom),
        );

        let animation_step = get_animation_step(wc);

        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) &&
                let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
                    let (tile_anim, anim_tileset) = map_data
                        .tile_anim_id
                        .and_then(|tile_anim_id| store.assets.tile_anims.get(&tile_anim_id))
                        .map(|tile_anim| (Some(tile_anim), store.assets.tilesets.get(&tile_anim.anim_tileset_id)))
                        .unwrap_or((None, None));
                    let map_rect = Self::get_map_rect(room_map, map_data);
                    self.draw_map_para_layer(ui, wc, &to_canvas, map_rect.min, map_data, tileset);
                    self.draw_map_bg_layer(ui, wc, &to_canvas, map_rect.min, map_data, tileset, tile_anim, anim_tileset, animation_step);
                }
        }

        // TODO: draw player sprite

        // draw map FG layer
        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) &&
                let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
                    let map_rect = Self::get_map_rect(room_map, map_data);
                    let (tile_anim, anim_tileset) = map_data
                        .tile_anim_id
                        .and_then(|tile_anim_id| store.assets.tile_anims.get(&tile_anim_id))
                        .map(|tile_anim| (Some(tile_anim), store.assets.tilesets.get(&tile_anim.anim_tileset_id)))
                        .unwrap_or((None, None));
                    self.draw_map_fg_layer(ui, wc, &to_canvas, map_rect.min, map_data, tileset, tile_anim, anim_tileset, animation_step);
                }
        }

        if wc.is_window_on_top(self.parent_window_id) {
            self.handle_keyboard(ui, canvas_rect.size(), room_size);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, store: &DataAssetStore) {
        if let Some(player_anim) = self.player_anim_id.and_then(|anim_id| store.assets.animations.get(&anim_id)) &&
            let Some(player_sprite) = store.assets.sprites.get(&player_anim.sprite_id) &&
            let Some(room) = self.room_id.and_then(|room_id| store.assets.rooms.get(&room_id)) {
                self.draw_frame(ui, wc, room, player_sprite, player_anim, store);
                wc.request_animation_repaint();
            } else {
                ui.label("Required assets not found!");
            }
    }

    fn handle_keyboard(&mut self, ui: &mut egui::Ui, canvas_size: Vec2, room_size: Vec2) {
        let (key_up, key_down, key_left, key_right) = ui.ctx().input(|i| {
            (
                i.key_down(egui::Key::ArrowUp),
                i.key_down(egui::Key::ArrowDown),
                i.key_down(egui::Key::ArrowLeft),
                i.key_down(egui::Key::ArrowRight)
            )
        });

        if key_up { self.scroll.y += 3.0; }
        if key_down { self.scroll.y -= 3.0; }
        if key_left { self.scroll.x += 3.0; }
        if key_right { self.scroll.x -= 3.0; }

        let screen_size = canvas_size / self.zoom;
        let min_scroll = screen_size - room_size;
        if min_scroll.x <= 0.0 { self.scroll.x = self.scroll.x.clamp(min_scroll.x, 0.0); }
        if min_scroll.y <= 0.0 { self.scroll.y = self.scroll.y.clamp(min_scroll.y, 0.0); }
    }
}
