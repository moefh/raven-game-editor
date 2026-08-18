
use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};
use crate::image::ImageCollection;

use super::super::{
    WindowContext,
    EditorStore,
};

pub trait PalSpriteFrameFixer {
    fn move_frame(&mut self, src_index: u32, dest_index: u32);

    fn add_pal_sprite_hole(&mut self, hole_start: u32, hole_size: u32, num_frames_after_hole: u32) {
        for index in (0..num_frames_after_hole).rev() {
            self.move_frame(hole_start + index, hole_start + hole_size + index);
        }
    }

    fn remove_pal_sprite_hole(&mut self, hole_start: u32, hole_size: u32, num_frames_after_hole: u32) {
        for index in 0..num_frames_after_hole {
            self.move_frame(hole_start + hole_size + index, hole_start + index);
        }
    }
}

fn reload_pal_sprite_texture(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    pal_sprite_id: DataAssetId,
) {
    if let Some(pal_sprite) = store.assets.pal_sprites.get_mut(&pal_sprite_id) {
        pal_sprite.load_texture(wc.tex_man, wc.egui.ctx, pal_sprite.texture_slot(false, false), true);
        pal_sprite.load_texture(wc.tex_man, wc.egui.ctx, pal_sprite.texture_slot(true, false), true);
    }
}

pub fn fix_after_pal_sprite_frames_added(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    pal_sprite_id: DataAssetId,
    hole_start: u32,
    hole_size: u32,
    num_frames_after_hole: u32
) {
    // fix pal sprite editors (undo/redo history)
    if let Some(pal_sprite_editor) = editors.pal_sprites.get_mut(&pal_sprite_id) {
        pal_sprite_editor.add_pal_sprite_hole(hole_start, hole_size, num_frames_after_hole);
    }

    reload_pal_sprite_texture(wc, store, pal_sprite_id);
}

pub fn fix_after_pal_sprite_frames_removed(
    wc: &mut WindowContext,
    store: &mut DataAssetStore,
    editors: &mut EditorStore,
    pal_sprite_id: DataAssetId,
    hole_start: u32,
    hole_size: u32,
    num_frames_after_hole: u32
) {
    // fix pal sprite editors (undo/redo history)
    if let Some(pal_sprite_editor) = editors.pal_sprites.get_mut(&pal_sprite_id) {
        pal_sprite_editor.remove_pal_sprite_hole(hole_start, hole_size, num_frames_after_hole);
    }

    reload_pal_sprite_texture(wc, store, pal_sprite_id);
}
