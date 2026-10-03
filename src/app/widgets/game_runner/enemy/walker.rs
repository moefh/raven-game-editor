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
use super::super::GameRunnerState;

pub struct Walker {}

impl Walker {
    pub fn new(enemy: &mut EnemyInfo, _room_id: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) -> Self {
        enemy.anim_loop = EnemyAnimLoop::Run;
        Walker {}
    }

    pub fn update(
        &mut self,
        enemy: &mut EnemyInfo,
        _room: &Room,
        _player: &Player,
        game_state: &GameRunnerState,
        anim: &SpriteAnimation,
        _store: &DataAssetStore,
    ) {
        enemy.walk_but_turn_on_bump_or_edge(enemy.direction.dx(), 0, game_state, anim);
    }
}
