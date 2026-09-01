use crate::data_asset::{
    DataAssetStore,
    Room,
    SpriteAnimation,
};

use super::controller::{*};
use super::collision::{*};

pub const DX_ACCEL: i32       =  0x100;
pub const DX_FRICTION: i32    =  0x0c0;
pub const DX_MAX: i32         =  0x700;

pub const DY_GRAVITY: i32     =  0x0c0;
pub const DY_MAX: i32         =  0x900;
pub const DY_JUMP_START: i32  = -0xa00;
pub const DY_JUMP_HOLD: i32   = -0x060;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Right,
    Left,
}

impl From<u8> for Direction {
    fn from(val: u8) -> Self {
        if val != 0 {
            Direction::Left
        } else {
            Direction::Right
        }
    }
}

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
        self.anim_frame = 0;
        self.anim_loop = 0;
        self.dx = 0;
        self.dy = 0;
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
        if self.state == PlayerState::Crouch && pad.held(GAMEPAD_UP|GAMEPAD_A) {
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
        if (self.state == PlayerState::Stand || self.state == PlayerState::Walk) && pad.pressed(GAMEPAD_A) {
            self.dy = DY_JUMP_START;
            self.state = PlayerState::Jump;
            self.anim_frame = 0;
        }

        // hold jump / start fall
        if self.state == PlayerState::Jump {
            if pad.held(GAMEPAD_A) && self.dy < 0 {
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
}
