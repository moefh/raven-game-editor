use crate::data_store::{
    DataStore,
    RoomCutscene,
    RoomCutsceneActionType,
};

use super::{
    GameAssets,
    GameRunnerState,
    GameRunnerAction,
    Player,
    PlayerState,
    Enemy,
    DarknessState,
};

struct ActionData<'a> {
    assets: &'a GameAssets<'a>,
    //store: &'a DataStore,
    state: &'a mut GameRunnerState,
    player: &'a mut Player,
    //enemies: &'a mut Vec<Enemy>,
    darkness: &'a mut DarknessState,
}

pub struct CutsceneState {
    pub cutscene_index: Option<usize>,
    pub action_index: usize,
    pub action_frame: u16,
    pub done: bool,
}

impl CutsceneState {
    pub fn new() -> Self {
        CutsceneState {
            cutscene_index: None,
            action_index: 0,
            action_frame: 0,
            done: false,
        }
    }

    pub fn clear(&mut self) {
        self.cutscene_index = None;
    }

    pub fn load(&mut self, cutscene_index: usize) {
        self.cutscene_index = Some(cutscene_index);
        self.action_index = 0;
        self.action_frame = 0;
        self.done = false;
    }

    fn run_action<'a>(
        &mut self,
        cutscene: &RoomCutscene,
        data: &mut ActionData<'a>,
    ) -> bool {
        if let Some(action) = cutscene.actions.get(self.action_index) {
            match action.action_type {
                RoomCutsceneActionType::Unknown {..} => {
                    true
                }

                RoomCutsceneActionType::DispelDarkness { speed } => {
                    data.darkness.dispel(speed);
                    true
                }

                RoomCutsceneActionType::SetPlayerDarkness { radius, fall_off, off_x, off_y } => {
                    data.darkness.set_at_player(radius, fall_off, off_x, off_y);
                    true
                }

                RoomCutsceneActionType::MovePlayerDarkness { off_x, off_y, speed } => {
                    data.darkness.move_at_player(off_x, off_y, speed);
                    true
                }

                RoomCutsceneActionType::SetPlayerState { state } => {
                    data.player.set_state(PlayerState::from_value(state));
                    true
                }

                RoomCutsceneActionType::ActivateEffect { set_effect_id } => {
                    data.state.actions.push(GameRunnerAction::ActivateEffect { effect_id: set_effect_id });
                    true
                }

                RoomCutsceneActionType::WaitFrames { num_frames } => {
                    self.action_frame += 1;
                    self.action_frame >= num_frames
                }

                RoomCutsceneActionType::WaitPlayerAnimationEnd {} => {
                    if let Some(cur_loop) = data.player.char_anim.advance(data.assets.char_anim) {
                        // true if player animation ended
                        (data.player.char_anim.frame >> 8) as usize >= cur_loop.frame_indices.len()
                    } else {
                        true
                    }
                }

                RoomCutsceneActionType::SetHudEnabled { enabled } => {
                    data.state.hud_disabled = enabled == 0;
                    true
                }
            }
        } else {
            self.done = true;
            false
        }
    }

    pub fn tick_engine(
        &mut self,
        assets: &GameAssets,
        _store: &DataStore,
        state: &mut GameRunnerState,
        player: &mut Player,
        _enemies: &mut Vec<Enemy>,
        darkness: &mut DarknessState,
    ) -> bool {
        if let Some(cutscene) = self.cutscene_index.and_then(|ci| assets.room.cutscenes.get(ci)) {
            let mut data = ActionData {
                assets,
                //store,
                state,
                player,
                //enemies,
                darkness,
            };
            while ! self.done && self.run_action(cutscene, &mut data) {
                self.action_index += 1;
                self.action_frame = 0;
            }
            self.done
        } else {
            true
        }
    }
}
