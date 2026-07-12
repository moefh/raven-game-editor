use std::io::Result;

use super::{
    error,
    ValueDef,
    ValueDefStruct,
    ValueArray,
    ValueStruct,
    ProjectData,
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

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("frame_indices"), ValueDef::ArrayRef), // u8[]
        (String::from("sprite"), ValueDef::AssetRef),
        (String::from("collision"), ValueDef::Struct(ValueDefStruct::new(vec![
            (String::from("x"), ValueDef::U16),
            (String::from("y"), ValueDef::U16),
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
        ]))),
        (String::from("use_foot_frames"), ValueDef::I8),
        (String::from("foot_overlap"), ValueDef::I8),
        (String::from("loops"), ValueDef::StructArray(   // 20 loops
            ValueDefStruct::new(vec![
                (String::from("offset"), ValueDef::U16),
                (String::from("length"), ValueDef::U16),
            ])
        )),
    ])
}

pub fn build_loops(
    animation_name: &str,
    all_frame_indices: &[u8],
    frame_slices: &ValueArray<ValueStruct>,
    use_foot_frames: bool,
    project_data: &ProjectData
) -> Result<Vec<SpriteAnimationLoop>> {
    let mut loops = Vec::new();
    for (loop_index, frame_slice) in frame_slices.values.iter().enumerate() {
        let frame_slice_offset = frame_slice.get_u16("offset")?;
        let frame_slice_len = frame_slice.get_u16("length")?;

        let mut frame_indices = Vec::new();
        for frame_index in 0..frame_slice_len {
            let src_offset = (frame_slice_offset + if use_foot_frames { 2*frame_index } else { frame_index }) as usize;
            let check_index = src_offset + if use_foot_frames { 1 } else { 0 };
            if check_index >= all_frame_indices.len() {
                return error(
                    format!("invalid animation frame index: {} >= {}", check_index, all_frame_indices.len()),
                    frame_slices.pos
                );
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
    }
    Ok(loops)
}

fn conv_collision(collision: &ValueStruct) -> Result<Rect> {
    Ok(Rect {
        x: collision.get_u16("x")? as i32,
        y: collision.get_u16("y")? as i32,
        w: collision.get_u16("width")? as i32,
        h: collision.get_u16("height")? as i32,
    })
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<SpriteAnimation> {
    let frame_indices_array = asset_struct.get_array_ref("frame_indices")?;
    let sprite_ref = asset_struct.get_asset_ref("sprite")?;
    let collision = asset_struct.get_struct("collision")?;
    let use_foot_frames = asset_struct.get_i8("use_foot_frames")?;
    let foot_overlap = asset_struct.get_i8("foot_overlap")?;
    let loops = asset_struct.get_struct_array("loops")?;

    let name = project_data.extract_asset_name("sprite_animation_frames_", frame_indices_array)?;

    let frame_indices = frame_indices_array.get_u8_array(project_data)?;
    let loops = build_loops(name, frame_indices, loops, use_foot_frames != 0, project_data)?;

    Ok(SpriteAnimation {
        asset: DataAsset::new(DataAssetType::SpriteAnimation, asset_id, DataAsset::identifier_to_name(name)),
        sprite_id: sprite_ref.get_asset_id(project_data)?,
        clip_rect: conv_collision(collision)?,
        foot_overlap,
        loops,
    })
}
