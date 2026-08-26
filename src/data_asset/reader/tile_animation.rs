use std::io::Result;

use super::{
    err,
    error,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    TileAnimation,
    TileAnimationLoop,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("parent_tileset"), ValueDef::AssetRef),
        (String::from("anim_tileset"), ValueDef::AssetRef),
        (String::from("loops"), ValueDef::StructArray(  // 256 loops
            ValueDefStruct::new(vec![
                (String::from("start"), ValueDef::U8),
                (String::from("len"), ValueDef::U8),
            ])
        ))
    ])
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<TileAnimation> {
    let parent_tileset_ref = asset_struct.get_asset_ref("parent_tileset")?;
    let anim_tileset_ref = asset_struct.get_asset_ref("anim_tileset")?;
    let loops_array = asset_struct.get_struct_array("loops")?;

    let name = project_data.get_asset_name_id(asset_id, DataAssetType::TileAnimation).ok_or_else(|| {
        err(format!("can't find tile animation {} in asset id enums definitions", asset_id), asset_struct.pos)
    })?;

    if loops_array.values.len() != TileAnimation::NUM_LOOPS {
        error(
            format!(
                "unexpected tile animation loops len: got {}, expected {}",
                loops_array.values.len(),
                TileAnimation::NUM_LOOPS
            ),
            loops_array.pos,
        )?;
    }
    let mut loops = [TileAnimationLoop::EMPTY; TileAnimation::NUM_LOOPS];
    for (index, tloop) in loops_array.values.iter().enumerate() {
        loops[index].start = tloop.get_u8("start")?;
        loops[index].len = tloop.get_u8("len")?;
    }

    Ok(TileAnimation {
        asset: DataAsset::new(DataAssetType::TileAnimation, asset_id, DataAsset::identifier_to_name(&name)),
        parent_tileset_id: parent_tileset_ref.get_asset_id(project_data)?,
        anim_tileset_id: anim_tileset_ref.get_asset_id(project_data)?,
        loops,
    })
}
