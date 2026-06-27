use std::io::Result;

use super::{
    error,
    Value,
    AssetDef,
    ProjectData,
    TokenPosition,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Rect,
    SpriteAnimation,
    SpriteAnimationFrame,
    SpriteAnimationLoop,
};

pub fn build_loops(animation_name: &str, all_frame_indices: &[u8], frame_slices: &Vec<Value>,
                   use_foot_frames: bool, project_data: &ProjectData, pos: TokenPosition) -> Result<Vec<SpriteAnimationLoop>> {
    let mut loops = Vec::new();
    for (loop_index, frame_slice_value) in frame_slices.iter().enumerate() {
        if let Value::Struct(value) = frame_slice_value && let [
            Value::U16(frame_slice_offset),
            Value::U16(frame_slice_len),
        ] = &value[..] {
            let mut frame_indices = Vec::new();
            for frame_index in 0..*frame_slice_len {
                let src_offset = (*frame_slice_offset + if use_foot_frames { 2*frame_index } else { frame_index }) as usize;
                let check_index = src_offset + if use_foot_frames { 1 } else { 0 };
                if check_index >= all_frame_indices.len() {
                    return error(format!("invalid animation frame index: {} >= {}",
                                         check_index, all_frame_indices.len()), pos);
                }
                let head_index = if all_frame_indices[src_offset] != 0xff {
                    Some(all_frame_indices[src_offset])
                } else {
                    None
                };
                let foot_index = if use_foot_frames && all_frame_indices[src_offset+1] != 0xff {
                    Some(all_frame_indices[src_offset+1])
                } else {
                    None
                };
                frame_indices.push(SpriteAnimationFrame { head_index, foot_index });
            }
            let name = project_data
                .get_asset_data_name(loop_index, "SPRITE_ANIMATION", animation_name, "LOOP")
                .unwrap_or_else(|| { format!("loop {}", loop_index) });
            loops.push(SpriteAnimationLoop {
                name_id: name,
                frame_indices,
                });
        } else {
            return error(format!("invalid animation loop data: {:?}", frame_slices), pos);
        }
    }
    Ok(loops)
}

fn conv_collision(values: &Vec<Value>, pos: TokenPosition) -> Result<Rect> {
    if let [
        Value::U16(x),
        Value::U16(y),
        Value::U16(w),
        Value::U16(h),
    ] = &values[..] {
        Ok(Rect {
            x: *x as i32,
            y: *y as i32,
            w: *w as i32,
            h: *h as i32,
        })
    } else {
        error(format!("bad sprite_animation collision data: {:?}", values), pos)
    }
}

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<SpriteAnimation> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::ArrayRef(frame_indices_array),
        Value::AssetRef(sprite_ref),
        Value::Struct(collision),
        Value::I8(use_foot_frames),
        Value::I8(foot_overlap),
        Value::Loop(loops),
    ] = &value[..] {
        let name = project_data.extract_asset_name("sprite_animation_frames_", frame_indices_array)?;

        let frame_indices = frame_indices_array.get_u8_array(project_data)?;
        let loops = build_loops(name, frame_indices, loops, *use_foot_frames != 0, project_data, asset_def.pos)?;

        Ok(SpriteAnimation {
            asset: DataAsset::new(DataAssetType::SpriteAnimation, asset_id, DataAsset::identifier_to_name(name)),
            sprite_id: sprite_ref.get_asset_id(project_data)?,
            clip_rect: conv_collision(collision, asset_def.pos)?,
            foot_overlap: *foot_overlap,
            loops,
        })
    } else {
        error(format!("bad sprite_animation data: {:?}", asset_def.value), asset_def.pos)
    }
}
