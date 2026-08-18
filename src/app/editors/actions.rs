use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};

use super::{
    fix_after_tileset_tiles_added,
    fix_after_tileset_tiles_removed,
    fix_after_sprite_frames_added,
    fix_after_sprite_frames_removed,
    fix_after_pal_sprite_frames_added,
    fix_after_pal_sprite_frames_removed,
    WindowContext,
    EditorStore,
};

pub enum EditorAction {
    TilesetTilesAdded { tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8 },
    TilesetTilesRemoved { tileset_id: DataAssetId, hole_start: u8, hole_size: u8, num_tiles_after_hole: u8 },
    SpriteFramesAdded { sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    SpriteFramesRemoved { sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    PalSpriteFramesAdded { pal_sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
    PalSpriteFramesRemoved { pal_sprite_id: DataAssetId, hole_start: u32, hole_size: u32, num_frames_after_hole: u32 },
}

impl EditorAction {
    pub fn run(self, wc: &mut WindowContext, store: &mut DataAssetStore, editors: &mut EditorStore) {
        match self {
            EditorAction::TilesetTilesAdded { tileset_id, hole_start, hole_size, num_tiles_after_hole } => {
                fix_after_tileset_tiles_added(wc, store, editors, tileset_id, hole_start, hole_size, num_tiles_after_hole);
            }

            EditorAction::TilesetTilesRemoved { tileset_id, hole_start, hole_size, num_tiles_after_hole } => {
                fix_after_tileset_tiles_removed(wc, store, editors, tileset_id, hole_start, hole_size, num_tiles_after_hole);
            }

            EditorAction::SpriteFramesAdded { sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_sprite_frames_added(wc, store, editors, sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::SpriteFramesRemoved { sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_sprite_frames_removed(wc, store, editors, sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::PalSpriteFramesAdded { pal_sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_pal_sprite_frames_added(wc, store, editors, pal_sprite_id, hole_start, hole_size, num_frames_after_hole);
            }

            EditorAction::PalSpriteFramesRemoved { pal_sprite_id, hole_start, hole_size, num_frames_after_hole } => {
                fix_after_pal_sprite_frames_removed(wc, store, editors, pal_sprite_id, hole_start, hole_size, num_frames_after_hole);
            }
        }
    }
}
