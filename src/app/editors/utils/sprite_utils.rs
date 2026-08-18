use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    SpriteAnimation,
};
use crate::image::ImageCollection;

use super::super::{
    WindowContext,
    EditorStore,
};

pub trait SpriteFrameFixer {
    fn move_frame(&mut self, src_index: u32, dest_index: u32);

    fn add_sprite_hole(&mut self, hole_start: u32, hole_size: u32, num_frames_after_hole: u32) {
        for index in (0..num_frames_after_hole).rev() {
            self.move_frame(hole_start + index, hole_start + hole_size + index);
        }
    }

    fn remove_sprite_hole(&mut self, hole_start: u32, hole_size: u32, num_frames_after_hole: u32) {
        for index in 0..num_frames_after_hole {
            self.move_frame(hole_start + hole_size + index, hole_start + index);
        }
    }
}

impl SpriteFrameFixer for SpriteAnimation {
    fn move_frame(&mut self, src_index: u32, dest_index: u32) {
        for aloop in self.loops.iter_mut() {
            for frame in aloop.frame_indices.iter_mut() {
                frame.head_index.iter_mut().for_each(|index| { if *index as u32 == src_index { *index = (dest_index & 0xff) as u8 } });
                frame.foot_index.iter_mut().for_each(|index| { if *index as u32 == src_index { *index = (dest_index & 0xff) as u8 } });
            }
        }
    }
}

fn reload_sprite_texture(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    sprite_id: DataAssetId,
) {
    if let Some(sprite) = store.assets.sprites.get_mut(&sprite_id) {
        sprite.load_texture(wc.tex_man, wc.egui.ctx, sprite.texture_slot(false, false), true);
        sprite.load_texture(wc.tex_man, wc.egui.ctx, sprite.texture_slot(true, false), true);
    }
}

pub fn fix_after_sprite_frames_added(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    sprite_id: DataAssetId,
    hole_start: u32,
    hole_size: u32,
    num_frames_after_hole: u32
) {
    // fix animations (frames) and animation editors (undo/redo history)
    for anim_id in store.asset_ids.animations.iter() {
        if let Some(anim) = store.assets.animations.get_mut(anim_id) && anim.sprite_id == sprite_id {
            anim.add_sprite_hole(hole_start, hole_size, num_frames_after_hole);
            if let Some(anim_editor) = editors.animations.get_mut(anim_id) {
                anim_editor.add_sprite_hole(hole_start, hole_size, num_frames_after_hole);
            }
        }
    }

    // fix sprite editors (undo/redo history)
    if let Some(sprite_editor) = editors.sprites.get_mut(&sprite_id) {
        sprite_editor.add_sprite_hole(hole_start, hole_size, num_frames_after_hole);
    }

    reload_sprite_texture(wc, store, sprite_id);
}

pub fn fix_after_sprite_frames_removed(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    sprite_id: DataAssetId,
    hole_start: u32,
    hole_size: u32,
    num_frames_after_hole: u32
) {
    // fix animations (frames) and animation editors (undo/redo history)
    for anim_id in store.asset_ids.animations.iter() {
        if let Some(anim) = store.assets.animations.get_mut(anim_id) && anim.sprite_id == sprite_id {
            anim.remove_sprite_hole(hole_start, hole_size, num_frames_after_hole);
            if let Some(anim_editor) = editors.animations.get_mut(anim_id) {
                anim_editor.remove_sprite_hole(hole_start, hole_size, num_frames_after_hole);
            }
        }
    }

    // fix sprite editors (undo/redo history)
    if let Some(sprite_editor) = editors.sprites.get_mut(&sprite_id) {
        sprite_editor.remove_sprite_hole(hole_start, hole_size, num_frames_after_hole);
    }

    reload_sprite_texture(wc, store, sprite_id);
}
