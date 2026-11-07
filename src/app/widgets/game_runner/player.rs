use crate::data_store::{
    DataStore,
    PlayerChar,
    Room,
    Tileset,
    SpriteAnimation,
    RoomTriggerType,
};

use super::{
    Direction,
    WindowContext,
    GameRunnerState,
    GameRunnerAction,
    Enemy,
    Bullet,
    Animation,
    AnimationDrawInfo,
};
use super::consts::{*};
use super::controller::{*};
use super::collision::{*};
use super::util::{*};

const JUMP_BUTTON: u32 = GAMEPAD_SNES_B;
const SHOOT_BUTTON: u32 = GAMEPAD_SNES_A;
const THROW_BUTTON: u32 = GAMEPAD_SNES_Y;
const INTERACT_BUTTON: u32 = GAMEPAD_SNES_X;

pub struct PlayerCharAssets<'a> {
    pub player_char: &'a PlayerChar,
    pub char_anim: &'a SpriteAnimation,
}

impl<'a> PlayerCharAssets<'a> {
    pub fn load(store: &'a DataStore) -> Option<Self> {
        let player_char_id = store.asset_ids.player_chars.get_first()?;
        let player_char = store.assets.player_chars.get(&player_char_id)?;
        let suit = player_char.suits.first()?;
        Some(PlayerCharAssets {
            player_char,
            char_anim: store.assets.animations.get(&suit.anim_id)?,
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayerState {
    Stand,
    Walk,
    Jump,
    Fall,
    Fall2,
    Crouch,
}

impl PlayerState {
    fn get_anim_loop(self, no_arm: bool) -> usize {
        let loop_index = match self {
            PlayerState::Stand  => { 0 }
            PlayerState::Walk   => { 1 }
            PlayerState::Jump   => { 2 }
            PlayerState::Fall   => { 3 }
            PlayerState::Fall2  => { 4 }
            PlayerState::Crouch => { 5 }
        };
        loop_index + if no_arm { 6 } else { 0 }
    }
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub dx: i32,              // 24.8 fixpoint
    pub dy: i32,              // 24.8 fixpoint
    pub state: PlayerState,
    pub direction: Direction,
    pub weapon_frames_left: u32,
    pub weapon: Option<usize>,
    pub shoot_wait: u32,
    pub shoot: bool,
    pub last_shot_frame: u32,
    pub char_anim: Animation,
    pub weapon_anim: Animation,
}

impl Player {
    pub fn new(room: &Room, assets: &PlayerCharAssets) -> Self {
        let (x, y, direction) = if let Some(player_spawn) = get_room_player_spawn(room) {
            let x = player_spawn.x as i32 + (4 * Tileset::TILE_SIZE as i32 - assets.char_anim.clip_rect.w) / 2;
            let y = player_spawn.y as i32 + 4 * Tileset::TILE_SIZE as i32 - assets.char_anim.clip_rect.h;
            let direction = if let RoomTriggerType::PlayerSpawn { direction } = player_spawn.trigger_type {
                direction.value().into()
            } else {
                Direction::Right
            };
            (x, y, direction)
        } else {
            (0, 0, Direction::Right)
        };

        Player {
            x,
            y,
            direction,
            dx: 0,
            dy: 0,
            state: PlayerState::Stand,
            char_anim: Animation::new(assets.char_anim.asset.id),
            weapon_anim: Animation::new(assets.char_anim.asset.id),
            weapon_frames_left: 0,
            weapon: None,
            shoot_wait: 0,
            shoot: false,
            last_shot_frame: 0,
        }
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
        self.char_anim.frame = 0;
    }

    pub fn control(&mut self, pad: &Controller, frame_counter: u32, assets: &PlayerCharAssets) {
        // weapons
        let weapon_index = if pad.held(THROW_BUTTON) { Some(0) } else if pad.held(SHOOT_BUTTON) { Some(1) } else { None };
        if let Some(weapon_index) = weapon_index && let Some(weapon) = assets.player_char.weapons.get(weapon_index) {
            self.weapon_frames_left = weapon.shoot.anim_len as u32;
            if self.weapon.is_none_or(|w| w != weapon_index) {
                self.weapon = Some(weapon_index);
                self.weapon_anim.sprite_anim_id = weapon.shoot.anim_id;
                self.weapon_anim.reset();
            }
            if self.shoot_wait == 0 && frame_counter.saturating_sub(self.last_shot_frame) > weapon.bullet.interval as u32 {
                if weapon.bullet.initial_delay as u32 > 0 {
                    self.shoot_wait = weapon.bullet.initial_delay as u32;
                } else {
                    self.shoot_wait = 1;  // will shoot immediatelly below
                }
            }
        }

        if self.shoot_wait > 0 {
            self.shoot_wait -= 1;
            if self.shoot_wait == 0 {
                self.last_shot_frame = frame_counter;
                self.shoot = true;
            }
        }

        // walk
        if self.state == PlayerState::Stand || self.state == PlayerState::Walk || self.state == PlayerState::Crouch {
            if self.state != PlayerState::Walk && pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.set_state(PlayerState::Walk);
            } else if self.state == PlayerState::Walk && ! pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.set_state(PlayerState::Stand);
            }
            if self.dx != 0 {
                if self.dx > 0 {
                    self.dx -= DX_FLOOR_FRICTION;
                    if self.dx < 0 { self.dx = 0; }
                } else {
                    self.dx += DX_FLOOR_FRICTION;
                    if self.dx > 0 { self.dx = 0; }
                }
            }
        } else if self.dx != 0 {
            if self.dx > 0 {
                self.dx -= DX_AIR_FRICTION;
                if self.dx < 0 { self.dx = 0; }
            } else {
                self.dx += DX_AIR_FRICTION;
                if self.dx > 0 { self.dx = 0; }
            }
        }

        // change direction
        if pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
            self.direction = if pad.held(GAMEPAD_RIGHT) { Direction::Right } else { Direction::Left };
        }

        // crouch
        if self.state == PlayerState::Stand && pad.held(GAMEPAD_DOWN) {
            self.set_state(PlayerState::Crouch);
        }
        if self.state == PlayerState::Crouch && pad.held(GAMEPAD_UP|JUMP_BUTTON) {
            self.set_state(PlayerState::Stand);
        }

        // limit x speed
        if pad.held(GAMEPAD_RIGHT) {
            self.dx += DX_ACCEL;
            if self.dx > DX_MAX { self.dx = DX_MAX; }
        }
        if pad.held(GAMEPAD_LEFT) {
            self.dx -= DX_ACCEL;
            if self.dx < -DX_MAX { self.dx = -DX_MAX; }
        }

        // jump
        if (self.state == PlayerState::Stand || self.state == PlayerState::Walk) && pad.pressed(JUMP_BUTTON) {
            self.dy = DY_JUMP_START;
            self.set_state(PlayerState::Jump);
        }

        // hold jump / start fall
        if self.state == PlayerState::Jump {
            if pad.held(JUMP_BUTTON) && self.dy < 0 {
                self.dy += DY_JUMP_HOLD;
            } else {
                self.set_state(PlayerState::Fall);
            }
        }

        // apply gravity
        if self.state != PlayerState::Stand && self.state != PlayerState::Walk && self.state != PlayerState::Crouch {
            self.dy += DY_GRAVITY;
        }
        if self.dy > DY_MAX {
            self.dy = DY_MAX;
        }
    }

    fn collect_items(
        &mut self,
        room: &Room,
        player_anim: &SpriteAnimation,
        store: &DataStore,
        pad: &Controller,
        game_state: &mut GameRunnerState,
        enemies: &mut Vec<Enemy>
    ) {
        for trigger in &room.triggers {
            match trigger.trigger_type {
                RoomTriggerType::WallButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                    let has_col_requirements = req_collectable_id == u16::MAX ||
                        game_state.acquired_collectable_ids.contains(&req_collectable_id);
                    let has_flag_requirements = req_flag_id == u16::MAX ||
                        game_state.acquired_flag_ids.contains(&req_flag_id);
                    if has_col_requirements && has_flag_requirements && pad.pressed(INTERACT_BUTTON) {
                        let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                        let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, width as i32, height as i32);
                        if player_rect.intersects_rect(&trigger_rect) {
                            game_state.activate_effect(set_effect_id, room, enemies, store);
                            if set_flag_id != u16::MAX && ! game_state.acquired_flag_ids.contains(&set_flag_id) {
                                game_state.spawn_enemies_for_flag(set_flag_id, room, enemies, store);
                                game_state.set_flag(set_flag_id, room, store);
                            }
                        }
                    }
                }
                RoomTriggerType::DetectorButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                    let has_col_requirements = req_collectable_id == u16::MAX ||
                        game_state.acquired_collectable_ids.contains(&req_collectable_id);
                    let has_flag_requirements = req_flag_id == u16::MAX ||
                        game_state.acquired_flag_ids.contains(&req_flag_id);
                    if has_col_requirements && has_flag_requirements {
                        let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                        let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, width as i32, height as i32);
                        if player_rect.intersects_rect(&trigger_rect) {
                            game_state.activate_effect(set_effect_id, room, enemies, store);
                            if set_flag_id != u16::MAX && ! game_state.acquired_flag_ids.contains(&set_flag_id){
                                game_state.spawn_enemies_for_flag(set_flag_id, room, enemies, store);
                                game_state.set_flag(set_flag_id, room, store);
                            }
                        }
                    }
                }
                RoomTriggerType::GetUpgrade { get_upgrade_id, req_flag_id } => {
                    let has_flag_requirements = req_flag_id == u16::MAX ||
                        game_state.acquired_flag_ids.contains(&req_flag_id);
                    if has_flag_requirements {
                        let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                        let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                        if player_rect.intersects_rect(&trigger_rect) {
                            game_state.get_upgrade(get_upgrade_id, room, store);
                        }
                    }
                }
                RoomTriggerType::GetCollectable { get_collectable_id, req_flag_id } if pad.pressed(INTERACT_BUTTON) => {
                    let has_flag_requirements = req_flag_id == u16::MAX ||
                        game_state.acquired_flag_ids.contains(&req_flag_id);
                    if has_flag_requirements {
                        let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                        let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                        if player_rect.intersects_rect(&trigger_rect) {
                            game_state.get_collectable(get_collectable_id, room, store);
                        }
                    }
                }
                RoomTriggerType::GetPickup { respawn_delay, sfx_id, .. } => {
                    let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                    let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                    if game_state.is_pickup_available(trigger.trigger_id) && player_rect.intersects_rect(&trigger_rect) {
                        game_state.get_pickup(trigger.trigger_id, respawn_delay, room, store);
                        if let Some(sfx_id) = sfx_id {
                            game_state.actions.push(GameRunnerAction::PlaySfx {
                                sfx_id,
                                volume: 0x100,
                                pitch_shift: 1<<10,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn collides_with_door(&self, anim: &SpriteAnimation, rect: &CollisionRect) -> bool {
        let px = self.x + anim.clip_rect.w/2;
        let py1 = self.y + 8;
        let py2 = self.y + anim.clip_rect.h - 9;
        rect.contains_point(px, py1) || rect.contains_point(px, py2)
    }

    fn create_shot(&self, crouch: bool, assets: &PlayerCharAssets) -> Option<Bullet> {
        let weapon = self.weapon.and_then(|index| assets.player_char.weapons.get(index))?;
        let sprite_anim_id = weapon.bullet.anim_id;
        let effect_anim_id = weapon.bullet.hit_anim_id;
        let hit_sfx_id = weapon.bullet.hit_sfx_id;
        let x = weapon.bullet.x as i32 + if crouch { weapon.shoot.crouch_x as i32 } else { 0 };
        let y = weapon.bullet.y as i32 + if crouch { weapon.shoot.crouch_y as i32 } else { 0 };
        let dx = weapon.bullet.dx as i32;
        let dy = weapon.bullet.dy as i32;
        let direction_dx = self.direction.dx();
        let damage = weapon.bullet.damage;
        Some(Bullet {
            x: (self.x + direction_dx * x) << 8,
            y: (self.y + y) << 8,
            dx: direction_dx * dx,
            dy,
            damage,
            anim: Animation::new(sprite_anim_id),
            effect_anim_id,
            hit_sfx_id,
        })
    }

    pub fn tick_engine(
        &mut self,
        room: &Room,
        assets: &PlayerCharAssets,
        store: &DataStore,
        pad: &Controller,
        game_state: &mut GameRunnerState,
        enemies: &mut Vec<Enemy>,
        bullets: &mut Vec<Bullet>,
    ) {
        let dx = self.dx >> 8;
        let dy = self.dy >> 8;
        let mut rect = CollisionRect {
            x: self.x,
            y: self.y,
            w: assets.char_anim.clip_rect.w,
            h: assets.char_anim.clip_rect.h,
        };
        let col_flags = collision_move(&mut rect, &game_state.room, dx, dy);
        if (col_flags & COLLISION_FLAGS_DOWN) != 0 {
            self.dy = 0;
            if pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.set_state(PlayerState::Walk);
            } else if self.state != PlayerState::Crouch {
                self.set_state(PlayerState::Stand);
            }
        } else if (col_flags & COLLISION_FLAGS_UP) != 0 {
            if self.dy < 0 { self.dy = 0; }
            if self.state == PlayerState::Jump {
                self.set_state(PlayerState::Fall);
            }
        } else if self.state == PlayerState::Stand || self.state == PlayerState::Walk || self.state == PlayerState::Crouch {
            let save_y = rect.y;
            if collision_move(&mut rect, &game_state.room, 0, 1) == 0 {
                self.set_state(PlayerState::Fall);
            }
            rect.y = save_y;
        }

        self.x = rect.x;
        self.y = rect.y;
        if self.weapon_frames_left != 0 {
            self.weapon_frames_left -= 1;
            if self.weapon_frames_left == 0 { self.weapon = None; }
        }
        if let Some(weapon_index) = self.weapon &&
            let Some(weapon) = assets.player_char.weapons.get(weapon_index) &&
            let Some(shoot_anim) = store.assets.animations.get(&weapon.shoot.anim_id) {
                self.weapon_anim.aloop = self.state.get_anim_loop(false);
                self.weapon_anim.advance(shoot_anim);
            }
        self.char_anim.aloop = self.state.get_anim_loop(self.weapon.is_some());
        if let Some(cur_loop) = self.char_anim.advance(assets.char_anim) &&
            self.state == PlayerState::Fall &&
            (self.char_anim.frame >> 8) as usize >= cur_loop.frame_indices.len() {
                // reaching end of animation in the "Fall" state moves to the "Fall2" state
                self.set_state(PlayerState::Fall2);
            }
        self.collect_items(room, assets.char_anim, store, pad, game_state, enemies);

        if self.shoot {
            self.shoot = false;
            if let Some(bullet) = self.create_shot(self.state == PlayerState::Crouch, assets) {
                bullets.push(bullet);
            }
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &DataStore,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let draw_info = AnimationDrawInfo {
            x: self.x,
            y: self.y,
            direction: self.direction,
        };
        self.char_anim.draw(ui, wc, &draw_info, store, screen_pos, zoom);
    }

    pub fn draw_weapon(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &DataStore,
        assets: &PlayerCharAssets,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let draw_info = AnimationDrawInfo {
            x: self.x,
            y: self.y,
            direction: self.direction,
        };
        if let Some(weapon) = self.weapon.and_then(|index| assets.player_char.weapons.get(index)) {
            self.weapon_anim.sprite_anim_id = weapon.shoot.anim_id;
            self.weapon_anim.draw(ui, wc, &draw_info, store, screen_pos, zoom);
        }
    }

}
