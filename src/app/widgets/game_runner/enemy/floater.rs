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

pub struct Floater {
    pub initial_y: i32,
    pub float_frame: usize,
}

impl Floater {
    const FLOAT_DY: &[i32] = &[0, -1, -2, -2, -1, 0, 1, 2, 2, 1 ];

    pub fn new(enemy: &mut EnemyInfo, _room: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) -> Self {
        enemy.anim_loop = EnemyAnimLoop::Float;
        Floater {
            initial_y: enemy.y,
            float_frame: 0,
        }
    }

    pub fn update(&mut self, enemy: &mut EnemyInfo, _room: &Room, _player: &Player, _anim: &SpriteAnimation, _store: &DataAssetStore) {
        self.float_frame += 16;
        if (self.float_frame >> 8) >= Self::FLOAT_DY.len() {
            self.float_frame &= 0xff;
        }
        enemy.y = self.initial_y + Self::FLOAT_DY[self.float_frame>>8];
    }
}
