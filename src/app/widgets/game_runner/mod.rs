mod player;
mod bullet;
mod enemy;
mod animation;
mod effect_display;
mod prompt;
mod util;
pub mod consts;
pub mod controller;
pub mod collision;

use std::collections::{
    HashSet,
    HashMap,
};

use egui::{
    Vec2,
    Rect,
    Pos2,
};

use crate::data_store::{
    DataStore,
    DataAssetId,
    SpriteAnimation,
    SpriteAnimationLoop,
    Sprite,
    Room,
    RoomMap,
    RoomTrigger,
    RoomTriggerType,
    RoomTriggerMapLayer,
    MapData,
    Tileset,
    PlayerChar,
    Font,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::super::{
    WindowContext,
};
use super::super::editors::{
    get_animation_step,
    get_game_runner_step,
    draw_layer,
    draw_room_layer,
    RoomSize,
    MapLayer,
    DrawMapInfo,
    DrawMapLayerData,
    DrawRoomLayerInfo,
    DrawRoomLayerData,
};

use animation::{*};
use controller::{*};
use collision::{*};
use player::{
    Player,
};
use enemy::{
    Enemy,
};
use bullet::{
    Bullet,
};
use effect_display::{
    EffectDisplay,
};
use prompt::{
    draw_room_trigger_prompts,
};
use util::{*};

pub const EMPTY_ANIMATION_LOOP: SpriteAnimationLoop = SpriteAnimationLoop {
    name_id: String::new(),
    frame_indices: Vec::new(),
    dont_loop: false,
    frame_speed: 64,
};

pub struct DrawInfo {
    pos: Pos2,
    zoom: f32,
}

pub struct GameAssets<'a> {
    pub room: &'a Room,
    pub font: &'a Font,
    pub gamepad_sprite: &'a Sprite,
    pub player_char: &'a PlayerChar,
    pub char_anim: &'a SpriteAnimation,
}

impl<'a> GameAssets<'a> {
    pub fn load(room_id: DataAssetId, store: &'a DataStore) -> Option<Self> {
        let room = store.assets.rooms.get(&room_id)?;
        let player_char_id = store.asset_ids.player_chars.get_first()?;
        let player_char = store.assets.player_chars.get(&player_char_id)?;
        let font = store.assets.fonts.get(&player_char.font_id)?;
        let gamepad_sprite = store.assets.sprites.get(&player_char.gamepad_sprite_id)?;
        let suit = player_char.suits.first()?;
        Some(GameAssets {
            room,
            font,
            gamepad_sprite,
            player_char,
            char_anim: store.assets.animations.get(&suit.anim_id)?,
        })
    }
}

struct RoomChangeInfo {
    src_door_index: usize,
}

pub enum GameRunnerAction {
    ActivateEffect { effect_id: u16 },
    SetFlag { flag_id: u16 },
    PlayAnimation { x: i32, y: i32, anim_id: DataAssetId, set_effect_id: u16, set_flag_id: u16 },
    PlaySfx { sfx_id: DataAssetId, volume: u16, pitch_shift: u16 },
    StopSfx,
}

pub struct GameRunnerRoomState {
    pub width: u32,
    pub height: u32,
    pub fx: Vec<u8>,
    pub bb: Vec<u8>,
    pub bg: Vec<u8>,
    pub fg: Vec<u8>,
}

impl GameRunnerRoomState {
    fn load_state_room(&mut self, game_assets: &GameAssets, store: &DataStore) {
        let room_size = RoomSize::from_room(game_assets.room, &store.assets.maps);
        let layer_len = (room_size.width * room_size.height) as usize;
        self.width = room_size.width;
        self.height = room_size.height;
        self.fx.clear();
        self.bb.clear();
        self.bg.clear();
        self.fg.clear();
        self.fx.resize(layer_len, 0xff);
        self.bb.resize(layer_len, 0xff);
        self.bg.resize(layer_len, 0xff);
        self.fg.resize(layer_len, 0xff);
        for room_map in &game_assets.room.maps {
            if let Some(map) = store.assets.maps.get(&room_map.map_id) {
                for y in 0..map.height {
                    let dest_start = ((y + room_map.y as u32) * room_size.width + room_map.x as u32) as usize;
                    let dest_end = dest_start + map.width as usize;
                    let src_start = (y * map.width) as usize;
                    let src_end = src_start + map.width as usize;
                    self.fx[dest_start..dest_end].copy_from_slice(&map.fx_tiles[src_start..src_end]);
                    self.bg[dest_start..dest_end].copy_from_slice(&map.bg_tiles[src_start..src_end]);
                    self.fg[dest_start..dest_end].copy_from_slice(&map.fg_tiles[src_start..src_end]);
                    if map.has_bb {
                        self.bb[dest_start..dest_end].copy_from_slice(&map.bb_tiles[src_start..src_end]);
                    }
                }
            }
        }
    }

    fn get_draw_layer_data<'a>(
        &'a self,
        room_map: &RoomMap,
        map_data: &MapData,
        layer: MapLayer,
        store: &'a DataStore
    ) -> Option<DrawMapLayerData<'a>> {
        let tileset = store.assets.tilesets.get(&map_data.tileset_id)?;
        let (tile_anim, anim_tileset) = if
            let Some(tile_anim) = map_data.tile_anim_id.as_ref().and_then(|tile_anim_id| store.assets.tile_anims.get(tile_anim_id)) &&
            let Some(anim_tileset) = store.assets.tilesets.get(&tile_anim.anim_tileset_id) {
                (Some(tile_anim), Some(anim_tileset))
            } else {
                (None, None)
            };

        let start = (room_map.y as u32 * self.width + room_map.x as u32) as usize;
        let (width, height, tiles) = match layer {
            MapLayer::Foreground => {
                (map_data.width, map_data.height, &self.fg[start..])
            }
            MapLayer::Background => {
                (map_data.width, map_data.height, &self.bg[start..])
            }
            MapLayer::Background2 => {
                (map_data.width, map_data.height, &self.bb[start..])
            }
            _ => { return None; }
        };

        Some(DrawMapLayerData {
            tileset,
            anim_tileset,
            tile_anim,
            layer,
            width,
            height,
            stride: self.width,
            tiles,
            fx_width: map_data.width,
            fx_height: map_data.height,
            fx_stride: self.width,
            fx_tiles: &self.fx[start..],
        })
    }
}

pub struct GameRunnerState {
    pub room_x: i32,
    pub room_y: i32,
    pub room: GameRunnerRoomState,
    pub activated_effect_ids: HashSet<u16>,
    pub acquired_upgrade_ids: HashSet<u16>,
    pub acquired_collectable_ids: HashSet<u16>,
    pub acquired_flag_ids: HashSet<u16>,
    pub acquired_pickups: HashMap<u16,u16>,   // trigger_id -> respawn_delay
    pub actions: Vec<GameRunnerAction>,
}

impl GameRunnerState {
    pub fn new() -> Self {
        GameRunnerState {
            room_x: 0,
            room_y: 0,
            activated_effect_ids: HashSet::new(),
            acquired_upgrade_ids: HashSet::new(),
            acquired_collectable_ids: HashSet::new(),
            acquired_flag_ids: HashSet::new(),
            acquired_pickups: HashMap::new(),
            actions: Vec::new(),
            room: GameRunnerRoomState {
                width: 0,
                height: 0,
                fx: Vec::new(),
                bb: Vec::new(),
                bg: Vec::new(),
                fg: Vec::new(),
            },
        }
    }

    pub fn clear(&mut self) {
        self.activated_effect_ids.clear();
        self.acquired_upgrade_ids.clear();
        self.acquired_collectable_ids.clear();
        self.acquired_flag_ids.clear();
        self.acquired_pickups.clear();
        self.actions.clear();
    }

    fn is_flag_set(&self, flag_id: u16) -> bool {
        self.acquired_flag_ids.contains(&flag_id)
    }

    fn is_effect_activated(&self, effect_id: u16) -> bool {
        self.activated_effect_ids.contains(&effect_id)
    }

    fn spawn_pickups(&mut self, game_assets: &GameAssets) {
        for trigger in &game_assets.room.triggers {
            if let RoomTriggerType::GetPickup { .. } = trigger.trigger_type {
                self.acquired_pickups.insert(trigger.trigger_id, 0);
            }
        }
    }

    pub fn get_upgrade(&mut self, upgrade_id: u16, _game_assets: &GameAssets, _store: &DataStore) {
        self.acquired_upgrade_ids.insert(upgrade_id);
    }

    pub fn get_collectable(&mut self, collectable_id: u16, _game_assets: &GameAssets, _store: &DataStore) {
        self.acquired_collectable_ids.insert(collectable_id);
    }

    pub fn is_pickup_available(&self, trigger_id: u16) -> bool {
        self.acquired_pickups.get(&trigger_id).copied().unwrap_or(u16::MAX) == 0
    }

    pub fn get_pickup(&mut self, trigger_id: u16, respawn: u16, _game_assets: &GameAssets, _store: &DataStore) {
        self.acquired_pickups.insert(trigger_id, if respawn == 0 { u16::MAX } else { respawn });
    }

    pub fn set_flag(&mut self, flag_id: u16, _game_assets: &GameAssets, _store: &DataStore) {
        if flag_id != u16::MAX {
            self.acquired_flag_ids.insert(flag_id);
        }
    }

    pub fn spawn_enemies_for_flag(&self, flag_id: u16, game_assets: &GameAssets, enemies: &mut Vec<Enemy>, store: &DataStore) {
        for trigger in &game_assets.room.triggers {
            if let RoomTriggerType::EnemySpawn { enemy_id, enemy_num, direction, req_flag_id } = trigger.trigger_type &&
                req_flag_id == flag_id &&
                let Some(enemy) = Enemy::new(enemy_num, (trigger.x, trigger.y), enemy_id, direction, game_assets, store) {
                    enemies.push(enemy);
                }
        }
    }

    pub fn activate_effect(&mut self, effect_id: u16, game_assets: &GameAssets, enemies: &mut Vec<Enemy>, store: &DataStore) {
        if effect_id == u16::MAX { return; }

        self.activated_effect_ids.insert(effect_id);

        const TILE_SIZE: i32 = Tileset::TILE_SIZE as i32;
        let room_width = self.room.width as i32;
        let room_height = self.room.height as i32;

        for trigger in &game_assets.room.triggers {
            match trigger.trigger_type {
                RoomTriggerType::UnblockTiles { width, height, on_effect_id } if on_effect_id == effect_id => {
                    let x_start = (trigger.x as i32 / TILE_SIZE).clamp(0, room_width);
                    let x_end = ((trigger.x as i32 + width as i32 - 1) / TILE_SIZE).clamp(0, room_width);
                    let y_start = (trigger.y as i32 / TILE_SIZE).clamp(0, room_height);
                    let y_end = ((trigger.y as i32 + height as i32 - 1) / TILE_SIZE).clamp(0, room_height);

                    for y in y_start ..= y_end {
                        for x in x_start ..= x_end {
                            self.room.fx[(y*room_width + x) as usize] |= 0x0f;
                        }
                    }
                }

                RoomTriggerType::ChangeTiles { map_index, tile_change_index, map_layer, on_effect_id } if on_effect_id==effect_id => {
                    let x_start = trigger.x as i32 / TILE_SIZE;
                    let y_start = trigger.y as i32 / TILE_SIZE;
                    if let Some(change) = game_assets.room.maps.get(map_index as usize)
                        .and_then(|room_map| store.assets.maps.get(&room_map.map_id))
                        .and_then(|map_data| map_data.tile_changes.get(tile_change_index as usize)) {
                            for cy in 0..change.height {
                                for cx in 0..change.width {
                                    if let Some(tile) = change.tiles[(cy * change.width + cx) as usize] {
                                        let x = cx as i32 + x_start;
                                        let y = cy as i32 + y_start;
                                        if x >= 0 && y >= 0 && x < room_width && y < room_height {
                                            let index = (y * room_width + x) as usize;
                                            match map_layer {
                                                RoomTriggerMapLayer::Background2 => { self.room.bb[index] = tile; }
                                                RoomTriggerMapLayer::Background => { self.room.bg[index] = tile; }
                                                RoomTriggerMapLayer::Foreground => { self.room.fg[index] = tile; }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                }

                RoomTriggerType::DisableAnimation { width, height, on_effect_id } if on_effect_id == effect_id => {
                    let x_start = (trigger.x as i32 / TILE_SIZE).clamp(0, room_width);
                    let x_end = ((trigger.x as i32 + width as i32 - 1) / TILE_SIZE).clamp(0, room_width);
                    let y_start = (trigger.y as i32 / TILE_SIZE).clamp(0, room_height);
                    let y_end = ((trigger.y as i32 + height as i32 - 1) / TILE_SIZE).clamp(0, room_height);

                    for y in y_start ..= y_end {
                        for x in x_start ..= x_end {
                            self.room.fx[(y*room_width + x) as usize] |= 0xf0;
                        }
                    }
                }

                RoomTriggerType::RemoveEnemy { enemy_num, on_effect_id } if on_effect_id == effect_id => {
                    enemies.retain(|e| e.id != enemy_num);
                }

                RoomTriggerType::PlayAnimation { anim_id, on_effect_id, set_effect_id, set_flag_id } if on_effect_id == effect_id => {
                    self.actions.push(GameRunnerAction::PlayAnimation {
                        x: trigger.x as i32,
                        y: trigger.y as i32,
                        anim_id,
                        set_effect_id,
                        set_flag_id,
                    });
                }

                RoomTriggerType::PlaySfx { sfx_id, volume, pitch_shift, on_effect_id, .. } if on_effect_id == effect_id => {
                    self.actions.push(GameRunnerAction::PlaySfx {
                        sfx_id,
                        volume,
                        pitch_shift,
                    });
                }

                RoomTriggerType::StopSfx { on_effect_id, .. } if on_effect_id == effect_id => {
                    self.actions.push(GameRunnerAction::StopSfx);
                }

                _ => {}
            }
        }
    }
}

struct ScreenUi {
    energy_sprite: Option<DataAssetId>,
}

impl ScreenUi {
    fn new() -> Self {
        ScreenUi {
            energy_sprite: None,
        }
    }

    fn setup(&mut self, store: &DataStore) {
        self.energy_sprite = store.assets.sprites
            .iter()
            .find(|s| s.asset.name == "screen/energy")
            .map(|spr| spr.asset.id);
    }
}

pub struct GameRunnerWidget {
    pub room_id: Option<DataAssetId>,
    pub frame_counter: u32,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub fx_display: Vec<EffectDisplay>,
    pub player: Option<Player>,
    pub controller: Controller,
    screen_ui: ScreenUi,
    state: GameRunnerState,
    map_animation_step: u32,
    last_game_runner_step: u32,
    required_assets_message: Option<String>,
}

impl GameRunnerWidget {
    pub const WIDTH: i32 = 320;
    pub const HEIGHT: i32 = 240;
    pub const SCREEN_SIZE: Vec2 = Vec2 { x: Self::WIDTH as f32, y: Self::HEIGHT as f32 };
    pub const SCREEN_HEADER_HEIGHT: i32 = 32;

    pub fn new() -> Self {
        GameRunnerWidget {
            required_assets_message: None,
            frame_counter: 0,
            screen_ui: ScreenUi::new(),
            player: None,
            enemies: Vec::new(),
            bullets: Vec::new(),
            fx_display: Vec::new(),
            controller: Controller::new(),

            room_id: None,
            map_animation_step: 0,
            last_game_runner_step: 0,
            state: GameRunnerState::new(),
        }
    }

    // ================================================
    // === ENGINE
    // ================================================

    fn place_player_at_door_exit(
        &mut self,
        door_exit: &RoomTrigger,
        dx: i32,
        dy: i32,
        player_anim: &SpriteAnimation
    ) {
        if let Some(player) = &mut self.player {
            const TILE_SIZE: i32 = Tileset::TILE_SIZE as i32;
            let tile_x = door_exit.x as i32 / TILE_SIZE;
            let tile_y = door_exit.y as i32 / TILE_SIZE;
            if get_room_tile_at(&self.state.room, tile_x + 1, tile_y) == 0x0f {
                player.x = dx + door_exit.x as i32 + TILE_SIZE + 2;
                if player.dx < 0 { player.dx = 0; }
            } else {
                player.x = dx + door_exit.x as i32 - 2 - player_anim.clip_rect.w;
                if player.dx > 0 { player.dx = 0; }
            }
            player.y = dy + door_exit.y as i32;
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

    fn process_room_triggers(&mut self, game_assets: &mut GameAssets) -> Option<RoomChangeInfo> {
        for (door_index, door) in game_assets.room.triggers.iter().enumerate() {
            if let RoomTriggerType::Door { req_flag_id, .. } = door.trigger_type {
                let player_anim = game_assets.char_anim;
                let has_flag_requirements = req_flag_id == u16::MAX || self.state.is_flag_set(req_flag_id);
                let rect = CollisionRect::new(door.x as i32, door.y as i32, Tileset::TILE_SIZE as i32, 4*Tileset::TILE_SIZE as i32);
                if has_flag_requirements &&
                    self.player.as_ref().map(|player| player.collides_with_door(player_anim, &rect)).unwrap_or(false) {
                        return Some(RoomChangeInfo { src_door_index: door_index });
                    }
            }
        }
        None
    }

    fn run_game_action(
        &mut self,
        wc: &mut WindowContext,
        action: &GameRunnerAction,
        game_assets: &GameAssets,
        store: &DataStore
    ) {
        match action {
            GameRunnerAction::ActivateEffect { effect_id } => {
                self.state.activate_effect(*effect_id, game_assets, &mut self.enemies, store);
            }
            GameRunnerAction::SetFlag { flag_id } => {
                self.state.set_flag(*flag_id, game_assets, store);
            }
            GameRunnerAction::PlayAnimation { anim_id, x, y, set_effect_id, set_flag_id } => {
                self.fx_display.push(
                    EffectDisplay::new(*x, *y, Direction::Right, *anim_id).at_end(*set_effect_id, *set_flag_id)
                );
            }
            GameRunnerAction::PlaySfx { sfx_id, volume, pitch_shift, .. } => {
                if let Some(sfx) = store.assets.sfxs.get(sfx_id) {
                    wc.sound_player.play_s16(
                        &sfx.samples,
                        44100.0 * *pitch_shift as f32 / (1<<12) as f32,
                        *volume as f32 / 256.0
                    );
                }
            }
            GameRunnerAction::StopSfx => {
                // not implemented
            }
        }
    }

    fn tick_engine(
        &mut self,
        wc: &mut WindowContext,
        game_assets: &mut GameAssets,
        store: &DataStore
    ) -> Option<RoomChangeInfo> {
        self.state.actions.clear();

        for delay in self.state.acquired_pickups.values_mut() {
            if *delay != 0 && *delay != u16::MAX { *delay -= 1; }
        }
        if let Some(player) = &mut self.player {
            player.tick_engine(game_assets, store, &self.controller, &mut self.state, &mut self.enemies, &mut self.bullets);
            for enemy in self.enemies.iter_mut() {
                enemy.tick_engine(player, store, &mut self.state, game_assets);
            }
        }
        self.bullets.retain_mut(|bullet| {
            let collided = bullet.tick_engine(game_assets, &mut self.enemies, store, &mut self.state);
            if collided {
                if let Some(bullet_anim) = store.assets.animations.get(&bullet.anim.sprite_anim_id) &&
                    let Some(effect_anim) = bullet.effect_anim_id.and_then(|anim_id| store.assets.animations.get(&anim_id)) {
                        let direction = Direction::from_dx(bullet.dx);
                        let x = (bullet.x>>8) + bullet_anim.clip_rect.w/2 - effect_anim.clip_rect.w/2;
                        let y = (bullet.y>>8) + bullet_anim.clip_rect.h/2 - effect_anim.clip_rect.h/2;
                        self.fx_display.push(EffectDisplay::new(x, y, direction, effect_anim.asset.id));
                    }
                if let Some(sfx) = bullet.hit_sfx_id.and_then(|sfx_id| store.assets.sfxs.get(&sfx_id)) {
                    wc.sound_player.play_s16(&sfx.samples, 44100.0, 1.0);
                }
            }
            ! collided  // retain if not collided
        });
        self.fx_display.retain_mut(|fx_display| {
            ! fx_display.tick_engine(&mut self.state, store)
        });
        let room_change = self.process_room_triggers(game_assets);

        while let Some(action) = self.state.actions.pop() {
            self.run_game_action(wc, &action, game_assets, store);
        }

        room_change
    }

    pub fn reset(&mut self) {
        self.required_assets_message = None;
        self.state.room_x = 0;
        self.state.room_y = 0;
        self.frame_counter = 0;
        self.last_game_runner_step = 0;
        self.map_animation_step = 0;
        self.state.clear();
        self.player = None;
        self.enemies.clear();
        self.bullets.clear();
        self.fx_display.clear();
        self.room_id = None;
    }

    pub fn set_room(&mut self, room_id: Option<DataAssetId>, store: &DataStore) {
        self.reset();
        self.screen_ui.setup(store);
        self.room_id = room_id;
        if let Some(room_id) = room_id {
            self.required_assets_message = self.load_asset_ids(room_id, store).err();
        }
    }

    fn load_asset_ids(&mut self, room_id: DataAssetId, store: &DataStore) -> Result<(), String> {
        let game_assets = GameAssets::load(room_id, store)
            .ok_or_else(|| String::from("error loading player character assets"))?;
        self.load_room(&game_assets, store);
        self.player = Some(Player::new(&game_assets));
        Ok(())
    }

    fn load_room(&mut self, game_assets: &GameAssets, store: &DataStore) {
        self.room_id = Some(game_assets.room.asset.id);
        self.enemies.clear();
        self.bullets.clear();
        self.fx_display.clear();
        self.spawn_enemies(game_assets, store);
        self.state.clear();
        self.state.spawn_pickups(game_assets);
        self.state.room_x = 0;
        self.state.room_y = 0;
        self.state.room.load_state_room(game_assets, store);
    }

    fn spawn_enemies(&mut self, game_assets: &GameAssets, store: &DataStore) {
        for trigger in &game_assets.room.triggers {
            if let RoomTriggerType::EnemySpawn { enemy_id, enemy_num, direction, req_flag_id } = trigger.trigger_type {
                let has_flag_requirements = req_flag_id == u16::MAX || self.state.is_flag_set(req_flag_id);
                if has_flag_requirements &&
                    let Some(enemy) = Enemy::new(enemy_num, (trigger.x, trigger.y), enemy_id, direction, game_assets, store) {
                        self.enemies.push(enemy);
                    }
            }
        }
    }

    // ================================================
    // === DISPLAY
    // ================================================

    fn follow_player(&mut self, player_anim: &SpriteAnimation) {
        let height = Self::HEIGHT - Self::SCREEN_HEADER_HEIGHT;
        if let Some(player) = &self.player {
            self.state.room_x = player.x + (player_anim.clip_rect.w - Self::WIDTH) / 2;
            self.state.room_y = player.y + (player_anim.clip_rect.h - height) / 2;
        }
    }

    fn clip_scroll(&mut self, room_size: RoomSize) {
        let height = Self::HEIGHT - Self::SCREEN_HEADER_HEIGHT;
        if room_size.width as i32 >= Self::WIDTH {
            self.state.room_x = self.state.room_x.clamp(0, room_size.width as i32 - Self::WIDTH);
        } else {
            self.state.room_x = 0;
        }
        if room_size.height as i32 >= height {
            self.state.room_y = self.state.room_y.clamp(0, room_size.height as i32 - height);
        } else {
            self.state.room_y = 0;
        }
    }

    fn draw_sprite(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        sprite: &Sprite,
        sprite_pos: Vec2,
        draw_info: &DrawInfo,
    ) {
        let sprite_size = egui::Vec2::new(sprite.width as f32, sprite.height as f32);
        let sprite_uv = sprite.get_item_uv(0);

        let draw_rect = egui::Rect::from_min_size(
            draw_info.pos + draw_info.zoom * sprite_pos,
            draw_info.zoom * sprite_size
        );
        let texture = sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        egui::Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
    }

    fn draw_room_triggers(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        game_assets: &GameAssets,
        store: &DataStore,
        draw_info: &DrawInfo,
    ) {
        for trigger in &game_assets.room.triggers {
            let trigger_pos = Vec2::new(trigger.x as f32, trigger.y as f32);
            match trigger.trigger_type {
                RoomTriggerType::GetUpgrade { get_upgrade_id, req_flag_id } => {
                    let has_flag_requirements = req_flag_id == u16::MAX || self.state.is_flag_set(req_flag_id);
                    if has_flag_requirements &&
                        ! self.state.acquired_upgrade_ids.contains(&get_upgrade_id) &&
                        let Some(item) = store.tables.upgrade.items.get(get_upgrade_id as usize) &&
                        let Some(sprite) = store.assets.sprites.get(&item.sprite_id) {
                            self.draw_sprite(ui, wc, sprite, trigger_pos, draw_info);
                        }
                }

                RoomTriggerType::GetCollectable { get_collectable_id, req_flag_id, .. } => {
                    let has_flag_requirements = req_flag_id == u16::MAX || self.state.is_flag_set(req_flag_id);
                    if has_flag_requirements &&
                        ! self.state.acquired_collectable_ids.contains(&get_collectable_id) &&
                        let Some(item) = store.tables.collectable.items.get(get_collectable_id as usize) &&
                        let Some(sprite) = store.assets.sprites.get(&item.sprite_id) {
                            self.draw_sprite(ui, wc, sprite, trigger_pos, draw_info);
                        }
                }

                RoomTriggerType::GetPickup { get_pickup_id, .. } => {
                    if self.state.acquired_pickups.get(&trigger.trigger_id).unwrap_or(&u16::MAX) == &0 &&
                        let Some(item) = store.tables.pickup.items.get(get_pickup_id as usize) &&
                        let Some(sprite) = store.assets.sprites.get(&item.sprite_id) {
                            self.draw_sprite(ui, wc, sprite, trigger_pos, draw_info);
                        }
                }

                _ => {}
            }
        }
    }

    fn draw_room_bg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        game_assets: &GameAssets,
        store: &DataStore,
        draw_map_info: &DrawMapInfo,
        draw_room_info: &DrawRoomLayerInfo
    ) {
        if let Some(tileset_id) = game_assets.room.parallax.tileset_id && let Some(tileset) = store.assets.tilesets.get(&tileset_id) {
            let draw_room_layer_data = DrawRoomLayerData {
                tileset,
                slot: TextureSlot::Opaque,
                width: game_assets.room.parallax.width,
                height: game_assets.room.parallax.height,
                stride: game_assets.room.parallax.width,
                tiles: &game_assets.room.parallax.tiles,
            };
            draw_room_layer(ui, wc, draw_room_info, &draw_room_layer_data, None);
        }

        for room_map in game_assets.room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) {
                let map_pos = draw_map_info.zoom * TILE_SIZE * Pos2::new(room_map.x as f32, room_map.y as f32);
                let draw_info = draw_map_info.add_pos(map_pos);
                if let Some(draw_data) = self.state.room.get_draw_layer_data(room_map, map_data, MapLayer::Background2, store) {
                    draw_layer(ui, wc, &draw_info, &draw_data, None);
                }
                if let Some(draw_data) = self.state.room.get_draw_layer_data(room_map, map_data, MapLayer::Background, store) {
                    draw_layer(ui, wc, &draw_info, &draw_data, None);
                }
            }
        }
    }

    fn draw_room_fg(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        game_assets: &GameAssets,
        store: &DataStore,
        draw_map_info: &DrawMapInfo
    ) {
        for room_map in game_assets.room.maps.iter() {
            if let Some(map_data) = store.assets.maps.get(&room_map.map_id) {
                let map_pos = draw_map_info.zoom * TILE_SIZE * Pos2::new(room_map.x as f32, room_map.y as f32);
                let draw_info = draw_map_info.add_pos(map_pos);
                if let Some(draw_data) = self.state.room.get_draw_layer_data(room_map, map_data, MapLayer::Foreground, store) {
                    draw_layer(ui, wc, &draw_info, &draw_data, None);
                }
            }
        }
    }

    fn calc_parallax_pos(&self, room_x: i32, room_y: i32, room_size: RoomSize, game_assets: &GameAssets) -> egui::Vec2 {
        if game_assets.room.parallax.width == 0 || game_assets.room.parallax.height == 0 {
            return egui::Vec2::ZERO;
        }
        let self_height = Self::HEIGHT - Self::SCREEN_HEADER_HEIGHT;
        let room_w = room_size.width as i32 - Self::WIDTH;
        let room_h = room_size.height as i32 - self_height;
        let para_x = if room_w <= 0 { 0 } else {
            room_x * (game_assets.room.parallax.width as i32 * Tileset::TILE_SIZE as i32 - Self::WIDTH) / room_w
        };
        let para_y = if room_h <= 0 { 0 } else {
            room_y * (game_assets.room.parallax.height as i32 * Tileset::TILE_SIZE as i32 - self_height) / room_h
        };
        egui::Vec2::new(para_x as f32, para_y as f32)
    }

    fn draw_screen(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        game_assets: &GameAssets,
        store: &DataStore
    ) {
        let min_size = Self::SCREEN_SIZE.max(ui.available_size());
        let response = ui.allocate_response(min_size, egui::Sense::click());
        let canvas_rect = response.rect;

        let room_size = get_room_size(game_assets.room, store);
        let zoom = (canvas_rect.width() / Self::SCREEN_SIZE.x).min(canvas_rect.height() / Self::SCREEN_SIZE.y);
        let screen_size = zoom * Self::SCREEN_SIZE;
        let full_draw_info = DrawInfo {
            pos: canvas_rect.min + 0.5 * (canvas_rect.size() - screen_size),
            zoom,
        };
        let screen_rect = Rect::from_min_size(full_draw_info.pos, screen_size);
        ui.shrink_clip_rect(screen_rect);

        self.follow_player(game_assets.char_anim);
        self.clip_scroll(room_size);

        // room
        let room_top = zoom * Self::SCREEN_HEADER_HEIGHT as f32;
        let room_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x, screen_rect.min.y + room_top),
            screen_rect.max
        );
        let room_pos = full_draw_info.pos + Vec2::new(0.0, room_top);
        let draw_map_info = DrawMapInfo {
            zoom,
            pos: room_pos - zoom * Vec2::new(self.state.room_x as f32, self.state.room_y as f32),
            screen_rect: room_rect,
            animation_step: Some(self.map_animation_step),
        };
        let draw_info = DrawInfo {
            pos: draw_map_info.pos,
            zoom,
        };
        let draw_room_info = DrawRoomLayerInfo {
            zoom,
            pos: room_pos - zoom * self.calc_parallax_pos(self.state.room_x, self.state.room_y, room_size, game_assets),
            screen_rect: room_rect,
        };

        self.draw_room_bg(ui, wc, game_assets, store, &draw_map_info, &draw_room_info);
        if let Some(player) = &mut self.player {
            player.draw(ui, wc, store, &draw_info);
        }
        self.draw_room_triggers(ui, wc, game_assets, store, &draw_info);
        for bullet in self.bullets.iter_mut() {
            bullet.draw(ui, wc, store, &draw_info);
        }
        if let Some(player) = &mut self.player {
            player.draw_weapon(ui, wc, store, game_assets, &draw_info);
        }
        for enemy in self.enemies.iter_mut() {
            enemy.draw(ui, wc, store, draw_map_info.pos, zoom);
        }
        self.draw_room_fg(ui, wc, game_assets, store, &draw_map_info);
        if let Some(player) = &self.player {
            draw_room_trigger_prompts(ui, wc, &self.state, game_assets, player, store, &draw_info);
        }

        for fx_display in self.fx_display.iter_mut() {
            fx_display.draw(ui, wc, store, &draw_info);
        }

        // header
        let header_rect = Rect::from_min_max(
            screen_rect.min,
            Pos2::new(screen_rect.max.x, screen_rect.min.y + room_top)
        );
        ui.painter().rect_filled(header_rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0, 0, 0));
        if let Some(sprite) = self.screen_ui.energy_sprite.and_then(|id| store.assets.sprites.get(&id)) {
            let sprite_uv = sprite.get_item_uv(0);
            let sprite_size = egui::Vec2::new(sprite.width as f32, sprite.height as f32);
            let draw_rect = egui::Rect::from_min_size(
                full_draw_info.pos + full_draw_info.zoom * egui::Vec2::new(2.0, 0.0),
                full_draw_info.zoom * sprite_size
            );
            let texture = sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque);
            egui::Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, window_id: egui::Id, store: &DataStore) {
        let room_id = match self.room_id {
            None => { return; }
            Some(room_id) => { room_id }
        };

        let mut game_assets = match GameAssets::load(room_id, store) {
            Some(assets) => { assets }
            None => {
                let message = self.required_assets_message.as_deref().unwrap_or("Required assets not found");
                ui.label(message);
                return;
            }
        };

        if self.advance_frame_counter(wc) {
            if wc.is_window_on_top(window_id) {
                self.controller.update(ui, wc);
            }
            if let Some(player) = &mut self.player {
                player.control(&self.controller, self.frame_counter, &game_assets);
            }
            if let Some(room_change) = self.tick_engine(wc, &mut game_assets, store) &&
               let Some(door) = game_assets.room.triggers.get(room_change.src_door_index) &&
                let RoomTriggerType::Door { dest_room_id, dest_trigger_id, .. } = door.trigger_type &&
                let Some(dest_room) = store.assets.rooms.get(&dest_room_id) &&
                let Some(dest_trigger) = dest_room.triggers.iter().find(|tr| tr.trigger_id == dest_trigger_id) {
                    game_assets.room = dest_room;
                    self.load_room(&game_assets, store);
                    let (dx, dy) = self.player.as_ref().map(|player| {
                        (player.x - door.x as i32, player.y - door.y as i32)
                    }).unwrap_or_else(|| (0, 0));
                    self.place_player_at_door_exit(dest_trigger, dx, dy, game_assets.char_anim);
                    self.follow_player(game_assets.char_anim);
                    return;  // room changed; don't show this frame
                }
        }
        self.draw_screen(ui, wc, &game_assets, store);
        wc.request_game_run_repaint();
    }
}
