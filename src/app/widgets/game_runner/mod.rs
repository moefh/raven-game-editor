mod player;
mod enemy;
mod util;
pub mod consts;
pub mod controller;
pub mod collision;

use controller::{*};
use player::Player;
use enemy::Enemy;
use util::{*};

use egui::{
    Vec2,
    Rect,
    Pos2,
};

use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    SpriteAnimation,
    SpriteAnimationLoop,
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

pub const EMPTY_ANIMATION_LOOP: SpriteAnimationLoop = SpriteAnimationLoop {
    name_id: String::new(),
    frame_indices: Vec::new(),
    dont_loop: false,
    frame_speed: 64,
};

pub struct GameRunnerWidget {
    pub room_id: Option<DataAssetId>,
    pub room_x: i32,
    pub room_y: i32,
    pub frame_counter: u32,
    pub player: Player,
    pub enemies: Vec<Enemy>,
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
            enemies: Vec::new(),
            controller: Controller::new(),

            room_id: None,
            room_x: 0,
            room_y: 0,
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
        for enemy in self.enemies.iter_mut() {
            enemy.tick_engine(room, &self.player, store);
        }
    }

    pub fn reset(&mut self) {
        self.room_x = 0;
        self.room_y = 0;
        self.frame_counter = 0;
        self.last_game_runner_step = 0;
        self.map_animation_step = 0;
        self.player.reset();
        self.enemies.clear();
        self.room_id = None;
    }

    pub fn set_room(&mut self, room_id: Option<DataAssetId>, store: &DataAssetStore) {
        if let Some(room_id) = room_id && let Some(room) = store.assets.rooms.get(&room_id) {
            self.load_room(room, store);
        } else {
            self.reset();
        }
    }

    fn load_room(&mut self, room: &Room, store: &DataAssetStore) {
        self.reset();

        if let Some(anim) = get_sprite_animation_by_name(store, Self::PLAYER_ANIMATION) {
            self.room_id = Some(room.asset.id);
            self.player.reset();
            self.player.anim_id = Some(anim.asset.id);
            self.player.move_to_spawn(room, anim);
            self.spawn_enemies(room, store);
        }
    }

    fn spawn_enemies(&mut self, room: &Room, store: &DataAssetStore) {
        for trigger in &room.triggers {
            if let RoomTriggerType::EnemySpawn { animation_id, enemy_type, direction } =  trigger.trigger_type {
                self.enemies.push(Enemy::new(trigger.x, trigger.y, animation_id, enemy_type, direction, room, store));
            }
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
        self.player.draw(ui, wc, player_sprite, player_anim, draw_map_info.pos, zoom);
        for enemy in self.enemies.iter_mut() {
            enemy.draw(ui, wc, draw_map_info.pos, zoom, store);
        }
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

        if let Some(player_anim) = self.player.anim_id.and_then(|anim_id| store.assets.animations.get(&anim_id)) {
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
