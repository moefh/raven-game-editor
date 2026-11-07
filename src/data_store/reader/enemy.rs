use std::io::Result;

use super::{
    err,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Enemy,
    EnemyType,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("type"), ValueDef::U16),
        (String::from("animation"), ValueDef::AssetRef),
        (String::from("health"), ValueDef::U16),
    ])
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<Enemy> {
    let enemy_type_num = asset_struct.get_u16("type")?;
    let animation_ref = asset_struct.get_asset_ref("animation")?;
    let health = asset_struct.get_u16("health")?;

    let name = project_data.get_asset_name_id(asset_id, DataAssetType::Enemy).ok_or_else(|| {
        err("can't find enemy in asset id enums definitions", asset_struct.pos)
    })?;

    Ok(Enemy {
        asset: DataAsset::new(DataAssetType::Enemy, asset_id, DataAsset::identifier_to_name(&name)),
        enemy_type: EnemyType::from(enemy_type_num),
        anim_id: animation_ref.get_asset_id(project_data)?,
        health,
    })
}
