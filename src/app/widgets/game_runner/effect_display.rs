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
};

pub struct EffectDisplay {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub anim: Animation,
}

impl EffectDisplay {
    pub fn new(x: i32, y: i32, direction: Direction, sprite_anim_id: DataAssetId) -> Self {
        EffectDisplay {
            x,
            y,
            direction,
            anim: Animation::new(sprite_anim_id),
        }
    }

    pub fn tick_engine(&mut self, store: &DataStore) -> bool {
        if let Some(sprite_anim) = store.assets.animations.get(&self.anim.sprite_anim_id) &&
            let Some(cur_loop) = self.anim.advance(sprite_anim) {
                (self.anim.frame >> 8) as usize >= cur_loop.frame_indices.len()
            } else {
                false
            }
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
