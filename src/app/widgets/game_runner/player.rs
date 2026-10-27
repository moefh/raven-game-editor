use crate::data_store::{
    DataStore,
    DataAssetId,
    Room,
    Tileset,
    Sprite,
    SpriteAnimation,
    SpriteAnimationLoop,
    RoomTriggerType,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::{
    EMPTY_ANIMATION_LOOP,
    Direction,
    WindowContext,
    GameRunnerState,
    Enemy,
};
use super::consts::{*};
use super::controller::{*};
use super::collision::{*};
use super::util::{*};

const JUMP_BUTTON: u32 = GAMEPAD_SNES_B;
const SHOOT_BUTTON: u32 = GAMEPAD_SNES_A;
const THROW_BUTTON: u32 = GAMEPAD_SNES_Y;
const INTERACT_BUTTON: u32 = GAMEPAD_SNES_X;

pub struct PlayerAssetIds {
    pub bunny_anim_id: DataAssetId,
    pub throw_anim_id: DataAssetId,
    pub gun_anim_id: DataAssetId,
}

pub struct PlayerAssets<'a> {
    pub bunny_anim: &'a SpriteAnimation,
    pub throw_anim: &'a SpriteAnimation,
    pub gun_anim: &'a SpriteAnimation,
    pub bunny_sprite: &'a Sprite,
    pub throw_sprite: &'a Sprite,
    pub gun_sprite: &'a Sprite,
}

impl<'a> PlayerAssets<'a> {
    pub fn load(ids: &PlayerAssetIds, store: &'a DataStore) -> Option<Self> {
        let bunny_anim = store.assets.animations.get(&ids.bunny_anim_id)?;
        let throw_anim = store.assets.animations.get(&ids.throw_anim_id)?;
        let gun_anim = store.assets.animations.get(&ids.gun_anim_id)?;
        Some(PlayerAssets {
            bunny_anim,
            throw_anim,
            gun_anim,
            bunny_sprite: store.assets.sprites.get(&bunny_anim.sprite_id)?,
            throw_sprite: store.assets.sprites.get(&throw_anim.sprite_id)?,
            gun_sprite: store.assets.sprites.get(&gun_anim.sprite_id)?,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerWeapon {
    None,
    Throw,
    Gun,
}

impl PlayerWeapon {
    pub fn is_some(self) -> bool {
        self != PlayerWeapon::None
    }
}

pub struct PlayerAnim {
    pub aloop: usize,
    pub frame: u32,  // 24.8 fixpoint
}

impl PlayerAnim {
    pub fn new() -> Self {
        PlayerAnim {
            frame: 0,
            aloop: 0,
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.aloop = 0;
    }

    pub fn get_sprite_frame(&mut self, anim: &SpriteAnimation) -> u32 {
        let empty_loop = EMPTY_ANIMATION_LOOP;
        let cur_loop = anim.loops.get(self.aloop)
            .or_else(|| anim.loops.first())
            .unwrap_or(&empty_loop);
        let loop_frame = self.frame >> 8;
        let loop_frame = if loop_frame as usize >= cur_loop.frame_indices.len() {
            if cur_loop.dont_loop {
                let loop_frame = (cur_loop.frame_indices.len() as u32).saturating_sub(1);
                self.frame = loop_frame << 8;
                loop_frame
            } else {
                self.frame &= 0xff;
                0
            }
        } else {
            loop_frame
        };

        cur_loop.frame_indices.get(loop_frame as usize).and_then(|frame| frame.head_index).unwrap_or(0) as u32
    }

    pub fn advance<'a>(&mut self, state: PlayerState, no_arm: bool, anim: &'a SpriteAnimation) -> Option<&'a SpriteAnimationLoop> {
        self.aloop = state.get_anim_loop(no_arm);
        if let Some(cur_loop) = anim.loops.get(self.aloop) {
            self.frame += cur_loop.frame_speed as u32;
            Some(cur_loop)
        } else {
            None
        }
    }
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub dx: i32,              // 24.8 fixpoint
    pub dy: i32,              // 24.8 fixpoint
    pub state: PlayerState,
    pub direction: Direction,
    pub weapon_frames: u32,
    pub weapon: PlayerWeapon,
    pub bunny_anim: PlayerAnim,
    pub weapon_anim: PlayerAnim,
    pub asset_ids: Option<PlayerAssetIds>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            state: PlayerState::Stand,
            direction: Direction::Right,
            bunny_anim: PlayerAnim::new(),
            weapon_anim: PlayerAnim::new(),
            weapon_frames: 0,
            weapon: PlayerWeapon::None,
            asset_ids: None,
        }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.dx = 0;
        self.dy = 0;
        self.state = PlayerState::Stand;
        self.direction = Direction::Right;
        self.bunny_anim.reset();
        self.weapon_anim.reset();
        self.asset_ids = None;
    }

    pub fn set_assets(&mut self, bunny: &SpriteAnimation, throw: &SpriteAnimation, gun: &SpriteAnimation) {
        self.asset_ids = Some(PlayerAssetIds {
            bunny_anim_id: bunny.asset.id,
            throw_anim_id: throw.asset.id,
            gun_anim_id: gun.asset.id,
        });
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
        self.bunny_anim.frame = 0;
    }

    pub fn move_to_spawn(&mut self, room: &Room, anim: &SpriteAnimation) {
        if let Some(player_spawn) = get_room_player_spawn(room) {
            self.x = player_spawn.x as i32 + (4 * Tileset::TILE_SIZE as i32 - anim.clip_rect.w) / 2;
            self.y = player_spawn.y as i32 + 4 * Tileset::TILE_SIZE as i32 - anim.clip_rect.h;
            if let RoomTriggerType::PlayerSpawn { direction } = player_spawn.trigger_type {
                self.direction = direction.value().into();
            }
        }
    }

    pub fn control(&mut self, pad: &Controller) {
        // weapons
        if pad.held(SHOOT_BUTTON) {
            self.weapon_frames = 8;
            if self.weapon != PlayerWeapon::Gun {
                self.weapon = PlayerWeapon::Gun;
                self.weapon_anim.reset();
            }
        }
        if pad.pressed(THROW_BUTTON) {
            self.weapon_frames = 12;
            if self.weapon != PlayerWeapon::Throw {
                self.weapon = PlayerWeapon::Throw;
                self.weapon_anim.reset();
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
                RoomTriggerType::GetUpgrade { get_upgrade_id } => {
                    let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                    let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                    if player_rect.intersects_rect(&trigger_rect) {
                        game_state.get_upgrade(get_upgrade_id, room, store);
                    }
                }
                RoomTriggerType::GetCollectable { get_collectable_id } if pad.pressed(INTERACT_BUTTON) => {
                    let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                    let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                    if player_rect.intersects_rect(&trigger_rect) {
                        game_state.get_collectable(get_collectable_id, room, store);
                    }
                }
                RoomTriggerType::GetPickup { respawn_delay, .. } => {
                    let player_rect = CollisionRect::new(self.x, self.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
                    let trigger_rect = CollisionRect::new(trigger.x as i32, trigger.y as i32, 16, 16);
                    if player_rect.intersects_rect(&trigger_rect) {
                        game_state.get_pickup(trigger.trigger_id, respawn_delay, room, store);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn tick_engine(
        &mut self,
        room: &Room,
        assets: &PlayerAssets,
        store: &DataStore,
        pad: &Controller,
        game_state: &mut GameRunnerState,
        enemies: &mut Vec<Enemy>
    ) {
        let dx = self.dx >> 8;
        let dy = self.dy >> 8;
        let mut rect = CollisionRect {
            x: self.x,
            y: self.y,
            w: assets.bunny_anim.clip_rect.w,
            h: assets.bunny_anim.clip_rect.h,
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
        if self.weapon_frames != 0 {
            self.weapon_frames -= 1;
            if self.weapon_frames == 0 { self.weapon = PlayerWeapon::None; }
        }
        match self.weapon {
            PlayerWeapon::Throw => { self.weapon_anim.advance(self.state, false, assets.throw_anim); }
            PlayerWeapon::Gun => { self.weapon_anim.advance(self.state, false, assets.gun_anim); }
            PlayerWeapon::None => {}
        }
        if let Some(cur_loop) = self.bunny_anim.advance(self.state, self.weapon.is_some(), assets.bunny_anim) &&
            self.state == PlayerState::Fall &&
            (self.bunny_anim.frame >> 8) as usize >= cur_loop.frame_indices.len() {
                // reaching end of animation in the "Fall" state moves to the "Fall2" state
                self.set_state(PlayerState::Fall2);
            }
        self.collect_items(room, assets.bunny_anim, store, pad, game_state, enemies);
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        assets: &PlayerAssets,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let frame = self.bunny_anim.get_sprite_frame(assets.bunny_anim);
        self.draw_anim(ui, wc, assets.bunny_sprite, frame, assets.bunny_anim, screen_pos, zoom);

        match self.weapon {
            PlayerWeapon::Gun => {
                let frame = self.weapon_anim.get_sprite_frame(assets.gun_anim);
                self.draw_anim(ui, wc, assets.gun_sprite, frame, assets.gun_anim, screen_pos, zoom);
            }
            PlayerWeapon::Throw => {
                let frame = self.weapon_anim.get_sprite_frame(assets.throw_anim);
                self.draw_anim(ui, wc, assets.throw_sprite, frame, assets.gun_anim, screen_pos, zoom);
            }
            PlayerWeapon::None => {}
        }
    }

    fn draw_anim(
        &self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        sprite: &Sprite,
        sprite_frame: u32,
        anim: &SpriteAnimation,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let sprite_x = match self.direction {
            Direction::Left  => { self.x - (sprite.width as i32 - (anim.clip_rect.x + anim.clip_rect.w)) }
            Direction::Right => { self.x - anim.clip_rect.x }
        };
        let sprite_y = self.y - anim.clip_rect.y;

        let sprite_size = egui::Vec2::new(sprite.width as f32, sprite.height as f32);
        let sprite_uv = match self.direction {
            Direction::Left => {
                let uv = sprite.get_item_uv(sprite_frame);
                egui::Rect::from_min_max(
                    egui::Pos2::new(uv.max.x, uv.min.y),
                    egui::Pos2::new(uv.min.x, uv.max.y)
                )
            }
            Direction::Right => {
                sprite.get_item_uv(sprite_frame)
            }
        };

        let draw_rect = egui::Rect::from_min_size(
            screen_pos + zoom * egui::Vec2::new(sprite_x as f32, sprite_y as f32),
            zoom * sprite_size
        );
        let texture = sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        egui::Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
    }
}
