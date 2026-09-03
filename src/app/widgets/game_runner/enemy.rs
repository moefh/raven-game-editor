use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    Room,
    RoomEntityDirection,
    Sprite,
    SpriteAnimation,
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
//use super::consts::{*};
use super::collision::{*};

/*
pub enum EnemyAnimLoop {
    Stand,
    Run,
    Jump,
    Fall,
    Look,
    Blink,
    Splat,
    PrepJump,
}
*/

//pub const ENEMY_ANIM_LOOP_STAND: usize = 0;
pub const ENEMY_ANIM_LOOP_RUN: usize = 1;
//pub const ENEMY_ANIM_LOOP_JUMP: usize = 2;
//pub const ENEMY_ANIM_LOOP_FALL: usize = 3;
//pub const ENEMY_ANIM_LOOP_LOOK: usize = 4;
//pub const ENEMY_ANIM_LOOP_BLINK: usize = 5;
//pub const ENEMY_ANIM_LOOP_SPLAT: usize = 6;
//pub const ENEMY_ANIM_LOOP_PREP_JUMP: usize = 7;
//pub const ENEMY_ANIM_LOOP_EXPLODE: usize = 8;
pub const ENEMY_ANIM_LOOP_FLOAT: usize = 9;

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Walker(WalkerState),
    //Chiller(ChillerState),
    //Hopper(HopperState),
    Floater(FloaterState),
    Unknown,
}

impl EnemyState {
    pub fn from_type(enemy_type: u16) -> Self {
        match enemy_type {
            0 => { EnemyState::Walker(WalkerState::default()) }
            //1 => { EnemyState::Chiller(ChillerState::default()) }
            //2 => { EnemyState::Hopper(HopperState::Stand) }
            3 => { EnemyState::Floater(FloaterState::default()) }
            _ => { EnemyState::Unknown }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum WalkerState {
    #[default]
    Walk,
}

impl WalkerState {
    pub fn get_anim_loop(self) -> usize {
        ENEMY_ANIM_LOOP_RUN
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum FloaterState {
    #[default]
    Float,
}

impl FloaterState {
    pub fn get_anim_loop(self) -> usize {
        ENEMY_ANIM_LOOP_FLOAT
    }
}

/*
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ChillerState {
    #[default]
    Walk,
    Look,
    Blink,
}

impl ChillerState {
    pub fn get_anim_loop(self) -> usize {
        match self {
            ChillerState::Walk   => { ENEMY_ANIM_LOOP_RUN }
            ChillerState::Look   => { ENEMY_ANIM_LOOP_LOOK }
            ChillerState::Blink  => { ENEMY_ANIM_LOOP_BLINK }
        }
    }
}
*/

pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub state: EnemyState,
    pub direction: Direction,
    pub anim_id: DataAssetId,
    pub anim_loop: usize,
    pub anim_frame: u32,  // 24.8 fixpoint
    pub dx: i32,          // 24.8 fixpoint
    pub dy: i32,          // 24.8 fixpoint
}

// === floater ===========================================
impl Enemy {
    fn init_floater(&mut self, state: FloaterState, _room: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) {
        self.dx = 0;
        self.dy = self.y;
        self.anim_loop = state.get_anim_loop();
    }

    fn update_floater(&mut self, _state: FloaterState, _room: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) {
        self.dx += 16;
        if (self.dx >> 8) >= Self::FLOAT_DELTA.len() as i32 {
            self.dx &= 0xff;
        }

        self.y = self.dy + Self::FLOAT_DELTA[(self.dx>>8) as usize];
    }
}

// === waker ===========================================
impl Enemy {
    fn init_walker(&mut self, _state: WalkerState, _room_id: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) {
        // nothing to do
    }

    fn update_walker(&mut self, state: WalkerState, room: &Room, anim: &SpriteAnimation, store: &DataAssetStore) {
        let mut rect = CollisionRect {
            x: self.x + self.direction.dx() * anim.clip_rect.w,
            y: self.y,
            w: anim.clip_rect.w,
            h: anim.clip_rect.h,
        };
        if collision_move(&mut rect, room, &store.assets.maps, 0, 1) == 0 {
            self.direction = self.direction.flip();
            self.anim_frame = 0;
        } else {
            let mut rect = CollisionRect {
                x: self.x,
                y: self.y,
                w: anim.clip_rect.w,
                h: anim.clip_rect.h,
            };
            let col_flags = collision_move(&mut rect, room, &store.assets.maps, self.direction.dx(), 0);
            if col_flags != 0 && col_flags != COLLISION_FLAGS_RAMP {
                self.direction = self.direction.flip();
                self.anim_frame = 0;
            }
            self.x = rect.x;
            self.y = rect.y;
        }
        self.anim_loop = state.get_anim_loop();
    }
}

impl Enemy {
    const FLOAT_DELTA: &[i32] = &[0, -1, -2, -2, -1, 0, 1, 2, 2, 1 ];

    pub fn new(
        x: i16,
        y: i16,
        anim_id: DataAssetId,
        enemy_type: u16,
        direction: RoomEntityDirection,
        room: &Room,
        store: &DataAssetStore
    ) -> Self {
        let mut enemy = Enemy {
            x: x as i32,
            y: y as i32,
            state: EnemyState::from_type(enemy_type),
            direction: direction.into(),
            anim_id,
            anim_frame: 0,
            anim_loop: 0,
            dx: 0,
            dy: 0,
        };
        enemy.init_engine(room, store);
        enemy
    }

    pub fn tick_engine(&mut self, room: &Room, store: &DataAssetStore) {
        if let Some(anim) = store.assets.animations.get(&self.anim_id) {
            match self.state {
                EnemyState::Walker(state) => { self.update_walker(state, room, anim, store); }
                EnemyState::Floater(state) => { self.update_floater(state, room, anim, store); }
                //EnemyState::Chiller(state) => { self.update_chiller(state, room, anim, store); }
                //EnemyState::Hopper(state) => { self.update_hopper(state, room, anim, store); }
                EnemyState::Unknown => { self.anim_loop = 0; }
            }
            if let Some(cur_loop) = anim.loops.get(self.anim_loop) {
                self.anim_frame += cur_loop.frame_speed as u32;
            }
        }
    }

    fn init_engine(&mut self, room: &Room, store: &DataAssetStore) {
        if let Some(anim) = store.assets.animations.get(&self.anim_id) {
            match self.state {
                EnemyState::Walker(state) => { self.init_walker(state, room, anim, store); }
                EnemyState::Floater(state) => { self.init_floater(state, room, anim, store); }
                //EnemyState::Chiller(state) => { self.init_chiller(state, room, anim, store); }
                //EnemyState::Hopper(state) => { self.init_hopper(state, room, anim, store); }
                EnemyState::Unknown => { self.anim_loop = 0; }
            }
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
