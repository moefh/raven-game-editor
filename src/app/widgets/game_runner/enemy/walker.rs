use crate::data_asset::{
    DataAssetStore,
    Room,
    SpriteAnimation,
};

use super::{
    Player,
    EnemyInfo,
    EnemyAnimLoop,
};

use super::super::collision::{*};

pub struct Walker {}

impl Walker {
    pub fn new(enemy: &mut EnemyInfo, _room_id: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) -> Self {
        enemy.anim_loop = EnemyAnimLoop::Run;
        Walker {}
    }

    pub fn update(&mut self, enemy: &mut EnemyInfo, room: &Room, _player: &Player, anim: &SpriteAnimation, store: &DataAssetStore) {
        let mut rect = CollisionRect {
            x: enemy.x + enemy.direction.dx() * anim.clip_rect.w,
            y: enemy.y,
            w: anim.clip_rect.w,
            h: anim.clip_rect.h,
        };
        if collision_move(&mut rect, room, &store.assets.maps, 0, 1) == 0 {
            enemy.direction = enemy.direction.flip();
            enemy.anim_frame = 0;
        } else {
            let mut rect = CollisionRect {
                x: enemy.x,
                y: enemy.y,
                w: anim.clip_rect.w,
                h: anim.clip_rect.h,
            };
            let col_flags = collision_move(&mut rect, room, &store.assets.maps, enemy.direction.dx(), 0);
            if col_flags != 0 && col_flags != COLLISION_FLAGS_RAMP {
                enemy.direction = enemy.direction.flip();
                enemy.anim_frame = 0;
            }
            enemy.x = rect.x;
            enemy.y = rect.y;
        }
    }
}
