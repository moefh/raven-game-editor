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

use crate::image::{
    TextureSlot,
    ImageCollection,
};
use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    SpriteAnimation,
    SpriteAnimationLoop,
    Tileset,
    Sprite,
    Room,
    RoomTriggerType,
};

use super::super::{
    WindowContext,
};
use super::super::editors::{
    get_animation_step,
    get_game_runner_step,
    draw_para_layer,
    draw_bg_layer,
    draw_fg_layer,
    RoomSize,
    DrawMapLayerInfo,
};

const EMPTY_ANIMATION_LOOP: SpriteAnimationLoop = SpriteAnimationLoop {
    name_id: String::new(),
    frame_indices: Vec::new(),
    dont_loop: false,
    frame_speed: 64,
};

pub struct GameRunnerWidget {
    pub room_id: Option<DataAssetId>,
    pub player_anim_id: Option<DataAssetId>,
    pub room_x: i32,
    pub room_y: i32,
    pub frame_counter: u32,
    pub player: Player,
    pub controller: Controller,
    map_animation_step: u32,
    last_game_runner_step: u32,
}

impl GameRunnerWidget {
    pub const WIDTH: i32 = 320;
    pub const HEIGHT: i32 = 240;
    pub const SCREEN_SIZE: Vec2 = Vec2 { x: Self::WIDTH as f32, y: Self::HEIGHT as f32 };
    const PLAYER_ANIMATION: &str = "bunny";

    pub fn new() -> Self {
        GameRunnerWidget {
            frame_counter: 0,
            player: Player::new(),
            controller: Controller::new(),

            room_id: None,
            room_x: 0,
            room_y: 0,
            player_anim_id: None,
            map_animation_step: 0,
            last_game_runner_step: 0,
        }
    }

    // ================================================
    // === ENGINE
    // ================================================

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

    fn tick_engine(
        &mut self,
        room: &Room,
        _player_sprite: &Sprite,
        player_anim: &SpriteAnimation,
        store: &DataAssetStore
    ) {
        self.player.tick_engine(room, player_anim, store, &self.controller);
    }

    pub fn reset(&mut self) {
        self.room_x = 0;
        self.room_y = 0;
        self.frame_counter = 0;
        self.last_game_runner_step = 0;
        self.map_animation_step = 0;
        self.room_id = None;
    }

    pub fn set_room(&mut self, room_id: Option<DataAssetId>, store: &DataAssetStore) -> bool {
        self.reset();
        self.room_id = room_id;

        if let Some(room_id) = self.room_id && let Some(anim) = get_sprite_animation_by_name(store, Self::PLAYER_ANIMATION) {
            self.player_anim_id = Some(anim.asset.id);
            self.player.reset();
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

    // ================================================
    // === DISPLAY
    // ================================================

    fn follow_player(&mut self, player_anim: &SpriteAnimation) {
        self.room_x = self.player.x + (player_anim.clip_rect.w - Self::WIDTH) / 2;
        self.room_y = self.player.y + (player_anim.clip_rect.h - Self::HEIGHT) / 2;
    }

    fn clip_scroll(&mut self, room_size: RoomSize) {
        if room_size.width as i32 >= Self::WIDTH {
            self.room_x = self.room_x.clamp(0, room_size.width as i32 - Self::WIDTH);
        } else {
            self.room_x = 0;
        }
        if room_size.height as i32 >= Self::HEIGHT {
            self.room_y = self.room_y.clamp(0, room_size.height as i32 - Self::HEIGHT);
        } else {
            self.room_y = 0;
        }
    }

    fn draw_room_bg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        store: &DataAssetStore,
        draw_map_info: &DrawMapLayerInfo
    ) {
        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) {
                let map_pos = draw_map_info.zoom * TILE_SIZE * Pos2::new(room_map.x as f32, room_map.y as f32);
                let draw_bg_info = draw_map_info.add_pos(map_pos);
                draw_para_layer(ui, wc, map_data, &store.assets.tilesets, &store.assets.tile_anims, &draw_bg_info, None);
                draw_bg_layer(ui, wc, map_data, &store.assets.tilesets, &store.assets.tile_anims, &draw_bg_info, None);
            }
        }
    }

    fn draw_room_fg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        room: &Room,
        store: &DataAssetStore,
        draw_map_info: &DrawMapLayerInfo
    ) {
        for room_map in room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) {
                let map_pos = draw_map_info.zoom * TILE_SIZE * Pos2::new(room_map.x as f32, room_map.y as f32);
                let draw_fg_info = draw_map_info.add_pos(map_pos);
                draw_fg_layer(ui, wc, map_data, &store.assets.tilesets, &store.assets.tile_anims, &draw_fg_info, None);
            }
        }
    }

    fn draw_player(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        player_sprite: &Sprite,
        player_anim: &SpriteAnimation,
        screen_pos: egui::Pos2,
        zoom: f32,
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

        let draw_rect = egui::Rect::from_min_size(
            screen_pos + zoom * Vec2::new(sprite_x as f32, sprite_y as f32),
            zoom * sprite_size
        );
        let texture = player_sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
    }

    fn draw_screen(
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

        let room_size = get_room_size(room, store);
        let zoom = (canvas_rect.width() / Self::SCREEN_SIZE.x).min(canvas_rect.height() / Self::SCREEN_SIZE.y);
        let screen_size = zoom * Self::SCREEN_SIZE;
        let screen_pos = canvas_rect.min + 0.5 * (canvas_rect.size() - screen_size);
        let screen_rect = Rect::from_min_size(screen_pos, screen_size);
        ui.shrink_clip_rect(screen_rect);

        self.follow_player(player_anim);
        self.clip_scroll(room_size);

        let draw_map_info = DrawMapLayerInfo {
            zoom,
            pos: screen_pos - zoom * Vec2::new(self.room_x as f32, self.room_y as f32),
            animation_step: Some(self.map_animation_step),
        };

        self.draw_room_bg(ui, wc, room, store, &draw_map_info);
        self.draw_player(ui, wc, player_sprite, player_anim, draw_map_info.pos, zoom);
        self.draw_room_fg(ui, wc, room, store, &draw_map_info);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, window_id: egui::Id, store: &DataAssetStore) {
        let room = match self.room_id {
            None => { return; }
            Some(room_id) => {
                if let Some(room) = store.assets.rooms.get(&room_id) {
                    room
                } else {
                    self.room_id = None;
                    return;
                }
            }
        };

        if let Some(player_anim) = self.player_anim_id.and_then(|anim_id| store.assets.animations.get(&anim_id)) {
            if let Some(player_sprite) = store.assets.sprites.get(&player_anim.sprite_id) {
                if self.advance_frame_counter(wc) {
                    if wc.is_window_on_top(window_id) {
                        self.controller.update(ui, wc);
                    }
                    self.player.control(&self.controller);
                    self.tick_engine(room, player_sprite, player_anim, store);
                }
                self.draw_screen(ui, wc, room, player_sprite, player_anim, store);
                wc.request_game_run_repaint();
            } else {
                ui.label("Required assets not found!");
            }
        } else {
            ui.label(format!("Sprite animation '{}' doesn't exist!", Self::PLAYER_ANIMATION));
        }
    }
}
