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

pub enum State {
    Walk,
    Look,
    Blink,
}

impl State {
    pub fn get_anim_loop(&self) -> EnemyAnimLoop {
        match self {
            State::Walk => { EnemyAnimLoop::Run }
            State::Look => { EnemyAnimLoop::Look }
            State::Blink => { EnemyAnimLoop::Blink }
        }
    }
}

pub struct Chiller {
    state: State,
    wait: i16,
}

impl Chiller {
    pub fn new(enemy: &mut EnemyInfo, _room_id: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) -> Self {
        let state = State::Look;
        enemy.anim_loop = state.get_anim_loop();
        Chiller {
            state,
            wait: 0,
        }
    }

    fn to_state(&mut self, state: State, enemy: &mut EnemyInfo, wait: i16) {
        self.state = state;
        self.wait = wait;
        enemy.anim_frame = 0;
    }

    pub fn update(&mut self, enemy: &mut EnemyInfo, room: &Room, _player: &Player, anim: &SpriteAnimation, store: &DataAssetStore) {
        match self.state {
            State::Look => {
                if enemy.is_at_animation_end(anim) {
                    self.to_state(State::Walk, enemy, 2000);
                }
            }
            State::Walk => {
                self.wait -= 1;
                if self.wait <= 0 || enemy.walk_but_turn_on_bump_or_edge(enemy.direction.dx(), 0, room, anim, store) {
                    self.to_state(State::Blink, enemy, 0);
                }
            }
            State::Blink => {
                if enemy.is_at_animation_end(anim) {
                    self.to_state(State::Look, enemy, 0);
                }
            }
        }

        enemy.anim_loop = self.state.get_anim_loop();
    }
}
