use crate::data_store::{
    DataStore,
    DataAssetId,
};

use super::{
    WindowContext,
    AnimationDrawInfo,
    Animation,
    Direction,
    DrawInfo,
    GameRunnerState,
    GameRunnerAction,
};

pub struct EffectDisplay {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub anim: Animation,
    pub set_effect_id: u16,
    pub set_flag_id: u16,
}

impl EffectDisplay {
    pub fn new(x: i32, y: i32, direction: Direction, sprite_anim_id: DataAssetId) -> Self {
        EffectDisplay {
            x,
            y,
            direction,
            anim: Animation::new(sprite_anim_id),
            set_effect_id: u16::MAX,
            set_flag_id: u16::MAX,
        }
    }

    pub fn at_end(mut self, set_effect_id: u16, set_flag_id: u16) -> Self {
        self.set_effect_id = set_effect_id;
        self.set_flag_id = set_flag_id;
        self
    }

    pub fn tick_engine(
        &mut self,
        game_state: &mut GameRunnerState,
        store: &DataStore
    ) -> bool {
        let ended = if let Some(sprite_anim) = store.assets.animations.get(&self.anim.sprite_anim_id) &&
            let Some(cur_loop) = self.anim.advance(sprite_anim) {
                (self.anim.frame >> 8) as usize >= cur_loop.frame_indices.len()
            } else {
                true
            };
        if ended {
            if self.set_effect_id != u16::MAX {
                game_state.actions.push(GameRunnerAction::ActivateEffect { effect_id: self.set_effect_id });
            }
            if self.set_flag_id != u16::MAX {
                game_state.actions.push(GameRunnerAction::SetFlag { flag_id: self.set_flag_id });
            }
        }
        ended
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &DataStore,
        draw_info: &DrawInfo,
    ) {
        let draw_anim_info = AnimationDrawInfo {
            x: self.x,
            y: self.y,
            direction: self.direction,
        };
        self.anim.draw(ui, wc, &draw_anim_info, store, draw_info);
    }
}
