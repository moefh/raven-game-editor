use crate::data_store::{
    DataStore,
    DataAssetId,
    SpriteAnimation,
    SpriteAnimationLoop,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::{
    EMPTY_ANIMATION_LOOP,
    Direction,
    WindowContext,
    DrawInfo,
};

pub struct AnimationDrawInfo {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
}

pub struct Animation {
    pub sprite_anim_id: DataAssetId,
    pub aloop: usize,
    pub frame: u32,  // 24.8 fixpoint
}

impl Animation {
    pub fn new(sprite_anim_id: DataAssetId) -> Self {
        Animation {
            sprite_anim_id,
            frame: 0,
            aloop: 0,
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.aloop = 0;
    }

    fn get_sprite_frame(&mut self, anim: &SpriteAnimation) -> u32 {
        let empty_loop = EMPTY_ANIMATION_LOOP;
        let cur_loop = anim.loops.get(self.aloop)
            .or_else(|| anim.loops.first())
            .unwrap_or(&empty_loop);
        let loop_frame = self.frame >> 8;
        let loop_frame = if loop_frame as usize >= cur_loop.frame_indices.len() {
            if cur_loop.dont_loop {
                let loop_frame = (cur_loop.frame_indices.len() as u32).saturating_sub(1);
                self.frame = loop_frame << 8;
                loop_frame
            } else {
                self.frame &= 0xff;
                0
            }
        } else {
            loop_frame
        };

        cur_loop.frame_indices.get(loop_frame as usize).and_then(|frame| frame.head_index).unwrap_or(0) as u32
    }

    pub fn advance<'a>(&mut self, anim: &'a SpriteAnimation) -> Option<&'a SpriteAnimationLoop> {
        if let Some(cur_loop) = anim.loops.get(self.aloop) {
            self.frame += cur_loop.frame_speed as u32;
            Some(cur_loop)
        } else {
            None
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        draw: &AnimationDrawInfo,
        store: &DataStore,
        draw_info: &DrawInfo,
    ) {
        if let Some(anim) = store.assets.animations.get(&self.sprite_anim_id) && let Some(sprite) = store.assets.sprites.get(&anim.sprite_id) {
            let draw_x = match draw.direction {
                Direction::Left  => { draw.x - (sprite.width as i32 - (anim.clip_rect.x + anim.clip_rect.w)) }
                Direction::Right => { draw.x - anim.clip_rect.x }
            };
            let draw_y = draw.y - anim.clip_rect.y;
            let draw_frame = self.get_sprite_frame(anim);

            let sprite_size = egui::Vec2::new(sprite.width as f32, sprite.height as f32);
            let sprite_uv = match draw.direction {
                Direction::Left => {
                    let uv = sprite.get_item_uv(draw_frame);
                    egui::Rect::from_min_max(
                        egui::Pos2::new(uv.max.x, uv.min.y),
                        egui::Pos2::new(uv.min.x, uv.max.y)
                    )
                }
                Direction::Right => {
                    sprite.get_item_uv(draw_frame)
                }
            };

            let draw_rect = egui::Rect::from_min_size(
                draw_info.pos + draw_info.zoom * egui::Vec2::new(draw_x as f32, draw_y as f32),
                draw_info.zoom * sprite_size
            );
            let texture = sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
            egui::Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);
        }
    }
}
