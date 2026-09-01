mod player;
mod util;
pub mod controller;
pub mod collision;

use controller::{*};
use player::{*};
use util::{*};

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
    SpriteAnimationLoop,
    TileAnimation,
    Tileset,
    MapData,
    Sprite,
    Room,
    RoomMap,
    RoomTriggerType,
};

use super::super::{
    WindowContext,
};
use super::super::editors::{
    get_animated_tile,
    get_animation_step,
    get_game_runner_step,
    get_map_layer_tile,
    MapLayer,
};

const EMPTY_ANIMATION_LOOP: SpriteAnimationLoop = SpriteAnimationLoop {
    name_id: String::new(),
    frame_indices: Vec::new(),
    dont_loop: false,
    frame_speed: 64,
};

pub struct GameRunnerWidget {
    pub parent_window_id: egui::Id,
    pub zoom: f32,
    pub scroll: Vec2,
    pub room_id: Option<DataAssetId>,
    pub player_anim_id: Option<DataAssetId>,
    pub frame_counter: u32,
    pub player: Player,
    pub controller: Controller,
    map_animation_step: u32,
    last_game_runner_step: u32,
}

impl GameRunnerWidget {
    pub const WIDTH: f32 = 320.0;
    pub const HEIGHT: f32 = 240.0;
    pub const SCREEN_SIZE: Vec2 = Vec2 { x: Self::WIDTH, y: Self::HEIGHT };
    const PLAYER_ANIMATION: &str = "bunny";

    pub fn new(parent_window_id: egui::Id) -> Self {
        GameRunnerWidget {
            zoom: 2.0,
            scroll: Vec2::ZERO,
            frame_counter: 0,
            player: Player::new(),
            controller: Controller::new(),

            parent_window_id,
            room_id: None,
            player_anim_id: None,
            map_animation_step: 0,
            last_game_runner_step: 0,
        }
    }

    pub fn reset(&mut self, room_id: DataAssetId, store: &DataAssetStore) -> bool {
        self.scroll = Vec2::ZERO;
        self.frame_counter = 0;
        self.last_game_runner_step = 0;
        self.map_animation_step = 0;
        self.room_id = Some(room_id);

        self.player.reset();
        if let Some(anim) = get_sprite_animation_by_name(store, Self::PLAYER_ANIMATION) {
            self.player_anim_id = Some(anim.asset.id);
            if let Some(room) = store.assets.rooms.get(&room_id) && let Some(player_spawn) = get_room_player_spawn(room) {
                self.player.x = player_spawn.x as i32 + (4 * Tileset::TILE_SIZE as i32 - anim.clip_rect.w) / 2;
                self.player.y = player_spawn.y as i32 + 4 * Tileset::TILE_SIZE as i32 - anim.clip_rect.h;
                if let RoomTriggerType::PlayerSpawn { direction } = player_spawn.trigger_type {
                    self.player.direction = direction.value().into();
                }
            }
            true
        } else {
            self.player_anim_id = None;
            false
        }
    }

    fn advance_frame_counter(&mut self, wc: &WindowContext) -> bool {
        self.map_animation_step = get_animation_step(wc);
        let game_runner_step = get_game_runner_step(wc);
        if self.last_game_runner_step != game_runner_step {
            self.frame_counter += 1;
            self.last_game_runner_step = game_runner_step;
            true
        } else {
            false
        }
    }

    fn follow_player(&mut self, canvas_size: Vec2, _room_size: Vec2, player_anim: &SpriteAnimation) {
        let screen_size = canvas_size / self.zoom;
        self.scroll = Vec2::new(
            screen_size.x / 2.0 - (self.player.x + player_anim.clip_rect.w / 2) as f32,
            screen_size.y / 2.0 - (self.player.y + player_anim.clip_rect.h / 2) as f32,
        );
    }

    fn clip_scroll(&mut self, canvas_size: Vec2, room_size: Vec2) {
        let screen_size = canvas_size / self.zoom;
        let min_scroll = screen_size - room_size;
        if min_scroll.x <= 0.0 {
            self.scroll.x = self.scroll.x.clamp(min_scroll.x, 0.0);
        } else {
            self.scroll.x = 0.0;
        }
        if min_scroll.y <= 0.0 {
            self.scroll.y = self.scroll.y.clamp(min_scroll.y, 0.0);
        } else {
            self.scroll.y = 0.0;
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
                    self.map_animation_step
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
    ) {
        for y in 0..map_data.height {
            for x in 0..map_data.width {
                let tile = get_map_layer_tile(map_data, MapLayer::Foreground, x, y);
                if tile == MapData::NO_TILE { continue; }
                let (tile, use_tileset) = if let Some(new_tile) = get_animated_tile(
                    tile,
                    MapLayer::Foreground,
                    get_map_layer_tile(map_data, MapLayer::Animation, x, y),
                    tile_anim,
                    self.map_animation_step
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

    fn draw_room_bg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        store: &DataAssetStore,
        to_canvas: &RectTransform
    ) {
        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) &&
                let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
                    let (tile_anim, anim_tileset) = map_data
                        .tile_anim_id
                        .and_then(|tile_anim_id| store.assets.tile_anims.get(&tile_anim_id))
                        .map(|tile_anim| (Some(tile_anim), store.assets.tilesets.get(&tile_anim.anim_tileset_id)))
                        .unwrap_or((None, None));
                    let map_rect = Self::get_map_rect(room_map, map_data);
                    self.draw_map_para_layer(ui, wc, to_canvas, map_rect.min, map_data, tileset);
                    self.draw_map_bg_layer(ui, wc, to_canvas, map_rect.min, map_data, tileset, tile_anim, anim_tileset);
                }
        }
    }

    fn draw_room_fg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        store: &DataAssetStore,
        to_canvas: &RectTransform
    ) {
        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) &&
                let Some(tileset) = store.assets.tilesets.get(&map_data.tileset_id) {
                    let map_rect = Self::get_map_rect(room_map, map_data);
                    let (tile_anim, anim_tileset) = map_data
                        .tile_anim_id
                        .and_then(|tile_anim_id| store.assets.tile_anims.get(&tile_anim_id))
                        .map(|tile_anim| (Some(tile_anim), store.assets.tilesets.get(&tile_anim.anim_tileset_id)))
                        .unwrap_or((None, None));
                    self.draw_map_fg_layer(ui, wc, to_canvas, map_rect.min, map_data, tileset, tile_anim, anim_tileset);
                }
        }
    }

    fn draw_player(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        player_sprite: &Sprite,
        player_anim: &SpriteAnimation,
        to_canvas: &RectTransform
    ) {
        let empty_loop = EMPTY_ANIMATION_LOOP;
        let cur_loop = player_anim.loops.get(self.player.anim_loop)
            .or_else(|| player_anim.loops.first())
            .unwrap_or(&empty_loop);
        let loop_frame = self.player.anim_frame >> 8;
        let loop_frame = if loop_frame as usize >= cur_loop.frame_indices.len() {
            if cur_loop.dont_loop {
                let loop_frame = (cur_loop.frame_indices.len() as u32).saturating_sub(1);
                self.player.anim_frame = loop_frame << 8;
                loop_frame
            } else {
                self.player.anim_frame &= 0xff;
                0
            }
        } else {
            loop_frame
        };

        let sprite_frame = cur_loop.frame_indices.get(loop_frame as usize).and_then(|frame| frame.head_index).unwrap_or(0) as u32;
        let sprite_x = match self.player.direction {
            Direction::Left  => { self.player.x - (player_sprite.width as i32 - (player_anim.clip_rect.x + player_anim.clip_rect.w)) }
            Direction::Right => { self.player.x - player_anim.clip_rect.x }
        };
        let sprite_y = self.player.y - player_anim.clip_rect.y;

        let sprite_size = Vec2::new(player_sprite.width as f32, player_sprite.height as f32);
        let sprite_rect = Rect::from_min_size(Pos2::new(sprite_x as f32, sprite_y as f32), sprite_size);
        let sprite_uv = match self.player.direction {
            Direction::Left => {
                let uv = player_sprite.get_item_uv(sprite_frame);
                Rect::from_min_max(
                    Pos2::new(uv.max.x, uv.min.y),
                    Pos2::new(uv.min.x, uv.max.y)
                )
            }
            Direction::Right => {
                player_sprite.get_item_uv(sprite_frame)
            }
        };

        let draw_rect = to_canvas.transform_rect(sprite_rect);
        let texture = player_sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
    }

    fn draw_frame(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        player_sprite: &Sprite,
        player_anim: &SpriteAnimation,
        store: &DataAssetStore
    ) {
        let min_size = Self::SCREEN_SIZE.max(ui.available_size());
        let response = ui.allocate_response(min_size, egui::Sense::click());
        let canvas_rect = response.rect;
        ui.shrink_clip_rect(canvas_rect);

        let room_size = get_room_size_vec2(room, store);
        self.follow_player(canvas_rect.size(), room_size, player_anim);
        self.clip_scroll(canvas_rect.size(), room_size);

        let to_canvas = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Self::SCREEN_SIZE),
            Rect::from_min_size(canvas_rect.min + self.zoom * self.scroll, Self::SCREEN_SIZE * self.zoom),
        );
        self.draw_room_bg(ui, wc, room, store, &to_canvas);
        self.draw_player(ui, wc, player_sprite, player_anim, &to_canvas);
        self.draw_room_fg(ui, wc, room, store, &to_canvas);
    }

    fn tick_engine(
        &mut self,
        room: &Room,
        _player_sprite: &Sprite,
        player_anim: &SpriteAnimation,
        store: &DataAssetStore
    ) {
        self.player.tick_engine(room, player_anim, store, &self.controller);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, store: &DataAssetStore) {
        if let Some(player_anim) = self.player_anim_id.and_then(|anim_id| store.assets.animations.get(&anim_id)) &&
            let Some(player_sprite) = store.assets.sprites.get(&player_anim.sprite_id) &&
            let Some(room) = self.room_id.and_then(|room_id| store.assets.rooms.get(&room_id)) {
                if self.advance_frame_counter(wc) {
                    if wc.is_window_on_top(self.parent_window_id) {
                        self.controller.update(ui, wc);
                    }
                    self.player.control(&self.controller);
                    self.tick_engine(room, player_sprite, player_anim, store);
                }
                self.draw_frame(ui, wc, room, player_sprite, player_anim, store);
                wc.request_game_run_repaint();
            } else {
                ui.label("Required assets not found!");
            }
    }
}
