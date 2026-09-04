use crate::data_asset::{
    DataAssetStore,
    Room,
    SpriteAnimation,
};

use super::{
    Player,
    EnemyInfo,
    EnemyAnimLoop,
    Direction,
};
use super::super::consts::{*};
use super::super::collision::{*};

pub enum State {
    AmbushWait,
    AmbushPlunge,
    Splat,
    PatrolWalk,
    PatrolBlink,
    PatrolLook,
    PouncePrep,
    PounceJump,
}

impl State {
    pub fn get_anim_loop(&self) -> EnemyAnimLoop {
        match self {
            State::AmbushWait => { EnemyAnimLoop::Stand }
            State::AmbushPlunge => { EnemyAnimLoop::Fall }
            State::Splat => { EnemyAnimLoop::Splat }
            State::PatrolWalk => { EnemyAnimLoop::Run }
            State::PatrolBlink => { EnemyAnimLoop::Blink }
            State::PatrolLook => { EnemyAnimLoop::Look }
            State::PouncePrep => { EnemyAnimLoop::PrepJump }
            State::PounceJump => { EnemyAnimLoop::Jump }
        }
    }
}

fn check_player_seen(enemy: &EnemyInfo, player: &Player, enemy_anim: &SpriteAnimation, store: &DataAssetStore) -> Option<Direction> {
    if let Some(player_anim_id) = player.anim_id && let Some(player_anim) = store.assets.animations.get(&player_anim_id) {
        enemy.check_player_seen(player, enemy_anim, player_anim)
    } else {
        None
    }
}

pub struct Hopper {
    pub state: State,
    pub dx: i32,          // 24.8 fixpoint
    pub dy: i32,          // 24.8 fixpoint
    pub wait: i16,
}

impl Hopper {
    pub fn new(enemy: &mut EnemyInfo, _room: &Room, _anim: &SpriteAnimation, _store: &DataAssetStore) -> Self {
        let state = State::AmbushWait;
        enemy.anim_loop = state.get_anim_loop();
        Hopper {
            state,
            dx: 0,
            dy: 0,
            wait: 0,
        }
    }

    fn to_state(&mut self, state: State, enemy: &mut EnemyInfo, wait: i16) {
        self.state = state;
        self.wait = wait;
        enemy.anim_frame = 0;
    }

    pub fn update(&mut self, enemy: &mut EnemyInfo, room: &Room, player: &Player, anim: &SpriteAnimation, store: &DataAssetStore) {
        match self.state {
            State::AmbushWait => {
                if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                    self.to_state(State::AmbushPlunge, enemy, 0);
                    enemy.direction = dir;
                    self.dx = dir.dx() * 0x80;
                    self.dy = 0;
                }
            }

            State::AmbushPlunge => {
                self.dy += DY_GRAVITY;
                if self.dy >= DY_MAX { self.dy = DY_MAX; }
                if (enemy.move_by(self.dx>>8, self.dy>>8, anim, room, store) & COLLISION_FLAGS_DOWN) != 0 {
                    self.to_state(State::Splat, enemy, 0);
                    self.dy = 0;
                }
            }

            State::Splat => {
                if enemy.is_at_animation_end(anim) {
                    if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                        self.to_state(State::PouncePrep, enemy, 0);
                        enemy.direction = dir;
                    } else {
                        self.to_state(State::PatrolWalk, enemy, 2000);
                        self.dx = enemy.direction.dx() * 0x100;
                        self.dy = 0;
                    }
                }
            }

            State::PatrolWalk => {
                if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                    self.to_state(State::PouncePrep, enemy, 0);
                    enemy.direction = dir;
                } else {
                    self.wait -= 1;
                    if self.wait <= 0 || enemy.walk_but_turn_on_bump_or_edge(self.dx>>8, self.dy>>8, room, anim, store) {
                        self.to_state(State::PatrolBlink, enemy, 0);
                    }
                }
            }

            State::PatrolBlink => {
                if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                    self.to_state(State::PouncePrep, enemy, 0);
                    enemy.direction = dir;
                } else if enemy.is_at_animation_end(anim) {
                    self.to_state(State::PatrolLook, enemy, 0);
                }
            }

            State::PatrolLook => {
                if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                    self.to_state(State::PouncePrep, enemy, 0);
                    enemy.direction = dir;
                } else if enemy.is_at_animation_end(anim) {
                    self.to_state(State::PatrolWalk, enemy, 2000);
                    self.dx = enemy.direction.dx() * 0x100;
                    self.dy = 0;
                }
            }

            State::PouncePrep => {
                if enemy.is_at_animation_end(anim) {
                    self.to_state(State::PounceJump, enemy, 0);
                    if let Some(dir) = check_player_seen(enemy, player, anim, store) {
                        enemy.direction = dir;
                    }
                    self.dx = enemy.direction.dx() * 0x200;
                    self.dy = -0xa00;
                }
            }

            State::PounceJump => {
                self.dy += DY_GRAVITY;
                if self.dy >= DY_MAX { self.dy = DY_MAX; }
                if (enemy.move_by(self.dx>>8, self.dy>>8, anim, room, store) & COLLISION_FLAGS_DOWN) != 0 {
                    self.to_state(State::Splat, enemy, 0);
                    self.dy = 0;
                }
            }
        }

        enemy.anim_loop = self.state.get_anim_loop();
    }
}
