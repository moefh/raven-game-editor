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
    CollisionRect,
};

pub struct Bullet {
    pub x: i32,
    pub y: i32,
    pub dx: i32,         // 24.8 fixpoint
    pub dy: i32,         // 24.8 fixpoint
    pub damage: u16,
    pub anim: Animation,
    pub effect_anim_id: Option<DataAssetId>,
}

impl Bullet {
    pub fn new(
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
        damage: u16,
        sprite_anim_id: DataAssetId,
        effect_anim_id: Option<DataAssetId>
    ) -> Self {
        Bullet {
            x: x << 8,
            y: y << 8,
            dx,
            dy,
            damage,
            anim: Animation::new(sprite_anim_id),
            effect_anim_id,
        }
    }

    pub fn tick_engine(
        &mut self,
        _room: &Room,
        enemies: &mut Vec<Enemy>,
        store: &DataStore,
        game_state: &mut GameRunnerState
    ) -> bool {
        self.x += self.dx;
        self.y += self.dy;

        if let Some(sprite_anim) = store.assets.animations.get(&self.anim.sprite_anim_id) {
            let x = self.x >> 8;
            let y = self.y >> 8;

            let rect = CollisionRect::new(x, y, sprite_anim.clip_rect.w, sprite_anim.clip_rect.h);
            for enemy in enemies.iter_mut() {
                if enemy.info.health != 0 && let Some(e_anim) = store.assets.animations.get(&enemy.info.anim_id) {
                    let e_rect = CollisionRect::new(enemy.info.x, enemy.info.y, e_anim.clip_rect.w, e_anim.clip_rect.h);
                    if rect.intersects_rect(&e_rect) {
                        enemy.damage(self.damage);
                        return true;
                    }
                }
            }

            if room_tiles_block_point(&game_state.room, rect.x, rect.y) ||
                room_tiles_block_point(&game_state.room, rect.x + rect.w, rect.y) ||
                room_tiles_block_point(&game_state.room, rect.x, rect.y + rect.h) ||
                room_tiles_block_point(&game_state.room, rect.x + rect.w, rect.y + rect.h) {
                    return true;
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
