use crate::data_store::{
    DataStore,
    SpriteAnimation,
};

use super::{
    Player,
    EnemyInfo,
    EnemyAnimLoop,
    GameAssets,
};
use super::super::GameRunnerState;

pub struct Walker {}

impl Walker {
    pub fn new(enemy: &mut EnemyInfo, _game_assets: &GameAssets, _anim: &SpriteAnimation, _store: &DataStore) -> Self {
        enemy.anim_loop = EnemyAnimLoop::Run;
        Walker {}
    }

    pub fn update(
        &mut self,
        enemy: &mut EnemyInfo,
        _player: &Player,
        game_state: &GameRunnerState,
        _game_assets: &GameAssets,
        anim: &SpriteAnimation,
    ) {
        enemy.walk_but_turn_on_bump_or_edge(enemy.direction.dx(), 0, game_state, anim);
    }
}
