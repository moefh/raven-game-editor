use crate::data_store::{
    DataStore,
    DataAssetId,
    Room,
};

use super::{
    room_tiles_block_point,
    Direction,
    WindowContext,
    GameRunnerState,
    AnimationDrawInfo,
    Animation,
    Enemy,
};

pub struct Bullet {
    pub x: i32,
    pub y: i32,
    pub dx: i32,         // 24.8 fixpoint
    pub dy: i32,         // 24.8 fixpoint
    pub anim: Animation,
    pub effect_anim_id: Option<DataAssetId>,
}

impl Bullet {
    pub fn new(x: i32, y: i32, dx: i32, dy: i32, sprite_anim_id: DataAssetId, effect_anim_id: Option<DataAssetId>) -> Self {
        Bullet {
            x: x << 8,
            y: y << 8,
            dx,
            dy,
            anim: Animation::new(sprite_anim_id),
            effect_anim_id,
        }
    }

    pub fn tick_engine(&mut self, _room: &Room, _enemies: &[Enemy], store: &DataStore, game_state: &mut GameRunnerState) -> bool {
        self.x += self.dx;
        self.y += self.dy;

        if let Some(sprite_anim) = store.assets.animations.get(&self.anim.sprite_anim_id) {
            let x = self.x >> 8;
            let y = self.y >> 8;

            // TODO: check collision with enemies

            if room_tiles_block_point(&game_state.room, x, y) ||
                room_tiles_block_point(&game_state.room, x + sprite_anim.clip_rect.w, y) ||
                room_tiles_block_point(&game_state.room, x, y + sprite_anim.clip_rect.h) ||
                room_tiles_block_point(&game_state.room, x + sprite_anim.clip_rect.w, y + sprite_anim.clip_rect.h) {
                    return true
                }

            self.anim.advance(sprite_anim);
            false
        } else {
            true  // remove bullet if we can't find its sprite animation
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        store: &DataStore,
        screen_pos: egui::Pos2,
        zoom: f32,
    ) {
        let draw_info = AnimationDrawInfo {
            x: self.x >> 8,
            y: self.y >> 8,
            direction: Direction::from_dx(self.dx),
        };
        self.anim.draw(ui, wc, &draw_info, store, screen_pos, zoom);
    }
}
