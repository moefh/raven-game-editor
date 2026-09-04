mod walker;
mod chiller;
mod hopper;
mod floater;

use crate::data_asset::{
    DataAssetStore,
    DataAssetId,
    Room,
    RoomEnemyType,
    RoomEntityDirection,
    Sprite,
    SpriteAnimation,
    Tileset,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::{
    WindowContext,
    Direction,
    Player,
};
use super::collision::{*};

use walker::Walker;
use chiller::Chiller;
use hopper::Hopper;
use floater::Floater;

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum EnemyAnimLoop {
    Stand,
    Run,
    Jump,
    Fall,
    Look,
    Blink,
    Splat,
    PrepJump,
    Explode,
    Float,
}

impl EnemyAnimLoop {
    pub fn index(self) -> usize {
        match self {
            EnemyAnimLoop::Stand    => { 0 }
            EnemyAnimLoop::Run      => { 1 }
            EnemyAnimLoop::Jump     => { 2 }
            EnemyAnimLoop::Fall     => { 3 }
            EnemyAnimLoop::Look     => { 4 }
            EnemyAnimLoop::Blink    => { 5 }
            EnemyAnimLoop::Splat    => { 6 }
            EnemyAnimLoop::PrepJump => { 7 }
            EnemyAnimLoop::Explode  => { 8 }
            EnemyAnimLoop::Float    => { 9 }
        }
    }
}

enum EnemyBehavior {
    Walker(Walker),
    Chiller(Chiller),
    Hopper(Hopper),
    Floater(Floater),
    Unknown,
}

impl EnemyBehavior {
    pub fn new(enemy_type: RoomEnemyType, enemy: &mut EnemyInfo, room: &Room, store: &DataAssetStore) -> Self {
        if let Some(anim) = store.assets.animations.get(&enemy.anim_id) {
            match enemy_type {
                RoomEnemyType::Walker => { return EnemyBehavior::Walker(Walker::new(enemy, room, anim, store)) }
                RoomEnemyType::Chiller => { return EnemyBehavior::Chiller(Chiller::new(enemy, room, anim, store)) }
                RoomEnemyType::Hopper => { return EnemyBehavior::Hopper(Hopper::new(enemy, room, anim, store)) }
                RoomEnemyType::Floater => { return EnemyBehavior::Floater(Floater::new(enemy, room, anim, store)) }
                RoomEnemyType::Other(_) => {}
            }
        }
        enemy.anim_loop = EnemyAnimLoop::Stand;
        enemy.anim_frame = 0;
        EnemyBehavior::Unknown
    }

    pub fn tick(&mut self, enemy: &mut EnemyInfo, player: &Player, anim: &SpriteAnimation, room: &Room, store: &DataAssetStore) {
        match self {
            EnemyBehavior::Walker(walker) => { walker.update(enemy, room, player, anim, store); }
            EnemyBehavior::Chiller(chiller) => { chiller.update(enemy, room, player, anim, store); }
            EnemyBehavior::Hopper(hopper) => { hopper.update(enemy, room, player, anim, store); }
            EnemyBehavior::Floater(floater) => { floater.update(enemy, room, player, anim, store); }
            EnemyBehavior::Unknown => {}
        }
    }
}

pub struct EnemyInfo {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub anim_id: DataAssetId,
    pub anim_loop: EnemyAnimLoop,
    pub anim_frame: u32,  // 24.8 fixpoint
}

impl EnemyInfo {
    fn check_player_seen(&self, player: &Player, enemy_anim: &SpriteAnimation, player_anim: &SpriteAnimation) -> Option<Direction> {
        const TILE_SIZE: i32 = Tileset::TILE_SIZE as i32;

        let watched_area = CollisionRect {
            x: self.x - 5*TILE_SIZE,
            y: self.y - 2*TILE_SIZE,
            w: enemy_anim.clip_rect.w + 10*TILE_SIZE,
            h: enemy_anim.clip_rect.h + 10*TILE_SIZE,
        };
        let px = player.x + player_anim.clip_rect.w / 2;
        let py = player.y + player_anim.clip_rect.h / 2;
        if watched_area.contains_point(px, py) {
            Some(Direction::from_dx(px - (self.x + enemy_anim.clip_rect.w/2)))
        } else {
            None
        }
    }

    fn move_by(&mut self, dx: i32, dy: i32, anim: &SpriteAnimation, room: &Room, store: &DataAssetStore) -> u32 {
        let mut rect = CollisionRect {
            x: self.x,
            y: self.y,
            w: anim.clip_rect.w,
            h: anim.clip_rect.h,
        };
        let flags = collision_move(&mut rect, room, &store.assets.maps, dx, dy);
        self.x = rect.x;
        self.y = rect.y;
        flags
    }

    fn is_at_animation_end(&self, anim: &SpriteAnimation) -> bool {
        if let Some(cur_loop) = anim.loops.get(self.anim_loop.index()).or_else(|| anim.loops.first()) {
            let loop_frame = (self.anim_frame + cur_loop.frame_speed as u32) >> 8;
            loop_frame as usize >= cur_loop.frame_indices.len()
        } else {
            true
        }
    }

    fn walk_but_turn_on_bump_or_edge(&mut self, dx: i32, dy: i32, room: &Room, anim: &SpriteAnimation, store: &DataAssetStore) -> bool {
        let mut fall_rect = CollisionRect {
            x: self.x + self.direction.dx() * anim.clip_rect.w,
            y: self.y,
            w: anim.clip_rect.w,
            h: anim.clip_rect.h,
        };
        if collision_move(&mut fall_rect, room, &store.assets.maps, 0, 1) == 0 {
            self.direction = self.direction.flip();
            self.anim_frame = 0;
            return true;
        }

        let mut move_rect = CollisionRect {
            x: self.x,
            y: self.y,
            w: anim.clip_rect.w,
            h: anim.clip_rect.h,
        };
        let col_flags = collision_move(&mut move_rect, room, &store.assets.maps, dx, dy);
        self.x = move_rect.x;
        self.y = move_rect.y;
        if (col_flags & !COLLISION_FLAGS_RAMP) != 0 {
            self.direction = self.direction.flip();
            self.anim_frame = 0;
            true
        } else {
            false
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
        let sprite_frame = if let Some(cur_loop) = anim.loops.get(self.anim_loop.index()).or_else(|| anim.loops.first()) {
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
            cur_loop.frame_indices.get(loop_frame as usize).and_then(|frame| frame.head_index).unwrap_or(0) as u32
        } else {
            0
        };

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

pub struct Enemy {
    enemy: EnemyInfo,
    behavior: EnemyBehavior,
}

impl Enemy {
    pub fn new(
        x: i16,
        y: i16,
        anim_id: DataAssetId,
        enemy_type: RoomEnemyType,
        direction: RoomEntityDirection,
        room: &Room,
        store: &DataAssetStore
    ) -> Self {
        let mut enemy = EnemyInfo {
            x: x as i32,
            y: y as i32,
            direction: direction.into(),
            anim_id,
            anim_loop: EnemyAnimLoop::Stand,
            anim_frame: 0,
        };
        let behavior = EnemyBehavior::new(enemy_type, &mut enemy, room, store);

        Enemy {
            enemy,
            behavior,
        }
    }

    pub fn tick_engine(&mut self, room: &Room, player: &Player, store: &DataAssetStore) {
        if let Some(anim) = store.assets.animations.get(&self.enemy.anim_id) {
            self.behavior.tick(&mut self.enemy, player, anim, room, store);
            if let Some(cur_loop) = anim.loops.get(self.enemy.anim_loop.index()) {
                self.enemy.anim_frame += cur_loop.frame_speed as u32;
            }
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        screen_pos: egui::Pos2,
        zoom: f32,
        store: &DataAssetStore,
    ) {
        if let Some(anim) = store.assets.animations.get(&self.enemy.anim_id) &&
            let Some(sprite) = store.assets.sprites.get(&anim.sprite_id) {
                self.enemy.draw(ui, wc, sprite, anim, screen_pos, zoom);
            }
    }
}
