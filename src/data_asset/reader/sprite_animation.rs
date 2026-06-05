use std::io::Result;

use super::ReaderAssetIndex;
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    AssetIdCollection,
    Rect,
    SpriteAnimation,
    SpriteAnimationFrame,
    SpriteAnimationLoop,
};

#[derive(Clone)]
pub struct LoopFrameSlice {
    pub offset: u16,
    pub len: u16,
}

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub sprite_ref: ReaderAssetIndex,
    pub clip_rect: Rect,
    pub foot_overlap: i8,
    pub loops: Vec<SpriteAnimationLoop>,
}

impl CreationData {
    pub fn build_loops(all_frame_indices: Vec<u8>, frame_slices: Vec<LoopFrameSlice>, use_foot_frames: bool) -> Vec<SpriteAnimationLoop> {
        let mut loops = Vec::new();
        for frame_slice in frame_slices {
            let mut frame_indices = Vec::new();
            for frame_index in 0..frame_slice.len {
                let src_offset = (frame_slice.offset + if use_foot_frames { 2*frame_index } else { frame_index }) as usize;
                let head_index = if all_frame_indices[src_offset] != 0xff {
                    Some(all_frame_indices[src_offset])
                } else { None };
                let foot_index = if use_foot_frames && all_frame_indices[src_offset+1] != 0xff {
                    Some(all_frame_indices[src_offset+1])
                } else { None };
                frame_indices.push(SpriteAnimationFrame { head_index, foot_index });
            }
            loops.push(SpriteAnimationLoop {
                name_id: String::new(),
                frame_indices,
            });
        }
        loops
    }

    pub fn into_sprite_animation(self, asset_ids: &AssetIdCollection) -> Result<SpriteAnimation> {
        Ok(SpriteAnimation {
            asset: DataAsset::new(DataAssetType::SpriteAnimation, self.asset_id, self.name),
            sprite_id: self.sprite_ref.get_asset_id(&asset_ids.sprites)?,
            clip_rect: self.clip_rect,
            foot_overlap: self.foot_overlap,
            loops: self.loops,
        })
    }
}
