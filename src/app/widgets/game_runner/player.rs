use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    Room,
    Tileset,
    Sprite,
    SpriteAnimation,
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
};
use super::consts::{*};
use super::controller::{*};
use super::collision::{*};
use super::util::{*};

const JUMP_BUTTON: u32 = GAMEPAD_SNES_B;

#[derive(Clone, Copy, PartialEq)]
pub enum PlayerState {
    Stand,
    Walk,
    Jump,
    Fall,
    Crouch,
}

impl PlayerState {
    fn get_anim_loop(self) -> usize {
        match self {
            PlayerState::Stand  => { 0 }
            PlayerState::Walk   => { 1 }
            PlayerState::Jump   => { 2 }
            PlayerState::Fall   => { 3 }
            PlayerState::Crouch => { 4 }
        }
    }
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub state: PlayerState,
    pub direction: Direction,
    pub anim_id: Option<DataAssetId>,
    pub anim_loop: usize,
    pub anim_frame: u32,  // 24.8 fixpoint
    pub dx: i32,          // 24.8 fixpoint
    pub dy: i32,          // 24.8 fixpoint
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 0,
            y: 0,
            state: PlayerState::Stand,
            direction: Direction::Right,
            anim_id: None,
            anim_frame: 0,
            anim_loop: 0,
            dx: 0,
            dy: 0,
        }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.state = PlayerState::Stand;
        self.direction = Direction::Right;
        self.anim_id = None;
        self.anim_frame = 0;
        self.anim_loop = 0;
        self.dx = 0;
        self.dy = 0;
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
        // walk
        if self.state == PlayerState::Stand || self.state == PlayerState::Walk || self.state == PlayerState::Crouch {
            if self.state != PlayerState::Walk && pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.state = PlayerState::Walk;
                self.anim_frame = 0;
            } else if self.state == PlayerState::Walk && ! pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.state = PlayerState::Stand;
                self.anim_frame = 0;
            }
            if self.dx != 0 {
                if self.dx > 0 {
                    self.dx -= DX_FRICTION;
                    if self.dx < 0 { self.dx = 0; }
                } else {
                    self.dx += DX_FRICTION;
                    if self.dx > 0 { self.dx = 0; }
                }
            }
        }

        // change direction
        if pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
            self.direction = if pad.held(GAMEPAD_RIGHT) { Direction::Right } else { Direction::Left };
        }

        // crouch
        if self.state == PlayerState::Stand && pad.held(GAMEPAD_DOWN) {
            self.state = PlayerState::Crouch;
            self.anim_frame = 0;
        }
        if self.state == PlayerState::Crouch && pad.held(GAMEPAD_UP|JUMP_BUTTON) {
            self.state = PlayerState::Stand;
            self.anim_frame = 0;
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
            self.state = PlayerState::Jump;
            self.anim_frame = 0;
        }

        // hold jump / start fall
        if self.state == PlayerState::Jump {
            if pad.held(JUMP_BUTTON) && self.dy < 0 {
                self.dy += DY_JUMP_HOLD;
            } else {
                self.state = PlayerState::Fall;
                self.anim_frame = 0;
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

    pub fn tick_engine(&mut self, room: &Room, player_anim: &SpriteAnimation, store: &DataAssetStore, pad: &Controller) {
        let dx = self.dx >> 8;
        let dy = self.dy >> 8;
        let mut rect = CollisionRect {
            x: self.x,
            y: self.y,
            w: player_anim.clip_rect.w,
            h: player_anim.clip_rect.h,
        };
        let col_flags = collision_move(&mut rect, room, &store.assets.maps, dx, dy);
        if (col_flags & COLLISION_FLAGS_DOWN) != 0 {
            self.dy = 0;
            if pad.held(GAMEPAD_RIGHT|GAMEPAD_LEFT) {
                self.state = PlayerState::Walk;
            } else if self.state != PlayerState::Crouch {
                self.state = PlayerState::Stand;
            }
            self.anim_frame = 0;
        } else if (col_flags & COLLISION_FLAGS_UP) != 0 {
            if self.dy < 0 { self.dy = 0; }
            if self.state == PlayerState::Jump {
                self.state = PlayerState::Fall;
                self.anim_frame = 0;
            }
        } else if self.state == PlayerState::Stand || self.state == PlayerState::Walk || self.state == PlayerState::Crouch {
            let save_y = rect.y;
            if collision_move(&mut rect, room, &store.assets.maps, 0, 1) == 0 {
                self.state = PlayerState::Fall;
                self.anim_frame = 0;
            }
            rect.y = save_y;
        }

        self.x = rect.x;
        self.y = rect.y;
        self.anim_loop = self.state.get_anim_loop();
        if let Some(cur_loop) = player_anim.loops.get(self.anim_loop) {
            self.anim_frame += cur_loop.frame_speed as u32;
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        sprite: &Sprite,
        anim: &SpriteAnimation,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let empty_loop = EMPTY_ANIMATION_LOOP;
        let cur_loop = anim.loops.get(self.anim_loop)
            .or_else(|| anim.loops.first())
            .unwrap_or(&empty_loop);
        let loop_frame = self.anim_frame >> 8;
        let loop_frame = if loop_frame as usize >= cur_loop.frame_indices.len() {
            if cur_loop.dont_loop {
                let loop_frame = (cur_loop.frame_indices.len() as u32).saturating_sub(1);
                self.anim_frame = loop_frame << 8;
                loop_frame
            } else {
                self.anim_frame &= 0xff;
                0
            }
        } else {
            loop_frame
        };

        let sprite_frame = cur_loop.frame_indices.get(loop_frame as usize).and_then(|frame| frame.head_index).unwrap_or(0) as u32;
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
