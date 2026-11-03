use std::io::Result;

use super::{
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
    PlayerChar,
    PlayerCharSuit,
    PlayerCharWeapon,
    PlayerCharWeaponShoot,
    PlayerCharWeaponBullet,
};

pub fn get_asset_def() -> ValueDefStruct
{
    ValueDefStruct::new(vec![
        (String::from("num_suits"), ValueDef::U16),
        (String::from("num_weapons"), ValueDef::U16),
        (String::from("suits"), ValueDef::ArrayRef),     // PLAYER_CHAR_SUIT
        (String::from("weapons"), ValueDef::ArrayRef),   // PLAYER_CHAR_WEAPON
    ])
}

pub fn get_global_struct_defs() -> Vec<(String, ValueDefStruct)> {
    vec![
        (String::from("PLAYER_CHAR_SUIT"), ValueDefStruct::new(vec![
            (String::from("animation"), ValueDef::AssetRef),
            (String::from("req_upgrade_id"), ValueDef::U16),
        ])),
        (String::from("PLAYER_CHAR_WEAPON"), ValueDefStruct::new(vec![
            (String::from("shoot_anim"), ValueDef::AssetRef),
            (String::from("shoot_sfx"), ValueDef::AssetRef),
            (String::from("bullet_anim"), ValueDef::AssetRef),
            (String::from("bullet_hit_anim"), ValueDef::AssetRef),
            (String::from("bullet_hit_sfx"), ValueDef::AssetRef),
            (String::from("req_collectable_id"), ValueDef::U16),
            (String::from("bullet_ammo_pickup_id"), ValueDef::U16),
            (String::from("crouch_x"), ValueDef::I8),
            (String::from("crouch_y"), ValueDef::I8),
            (String::from("bullet_x"), ValueDef::I8),
            (String::from("bullet_y"), ValueDef::I8),
            (String::from("bullet_dx"), ValueDef::I16),
            (String::from("bullet_dy"), ValueDef::I16),
            (String::from("bullet_damage"), ValueDef::U16),
            (String::from("bullet_initial_delay"), ValueDef::U8),
            (String::from("bullet_interval"), ValueDef::U8),
            (String::from("shoot_anim_len"), ValueDef::U8),
        ])),
    ]
}

fn conv_suit(suit: &ValueStruct, name: String, project_data: &ProjectData) -> Result<PlayerCharSuit> {
    let animation_ref = suit.get_asset_ref("animation")?;
    let req_upgrade_id = suit.get_u16("req_upgrade_id")?;

    Ok(PlayerCharSuit {
        name,
        anim_id: animation_ref.get_asset_id(project_data)?,
        req_upgrade_id,
    })
}

fn conv_weapon(weapon: &ValueStruct, name: String, project_data: &ProjectData) -> Result<PlayerCharWeapon> {
    let shoot_anim_ref = weapon.get_asset_ref("shoot_anim")?;
    let shoot_sfx_ref = weapon.get_asset_ref("shoot_sfx")?;
    let bullet_anim_ref = weapon.get_asset_ref("bullet_anim")?;
    let bullet_hit_anim_ref = weapon.get_asset_ref("bullet_hit_anim")?;
    let bullet_hit_sfx_ref = weapon.get_asset_ref("bullet_hit_sfx")?;

    let req_collectable_id = weapon.get_u16("req_collectable_id")?;
    let ammo_pickup_id = weapon.get_u16("bullet_ammo_pickup_id")?;
    let shoot_anim_len = weapon.get_u8("shoot_anim_len")?;
    let bullet_initial_delay = weapon.get_u8("bullet_initial_delay")?;
    let bullet_interval = weapon.get_u8("bullet_interval")?;
    let crouch_x = weapon.get_i8("crouch_x")?;
    let crouch_y = weapon.get_i8("crouch_y")?;
    let bullet_x = weapon.get_i8("bullet_x")?;
    let bullet_y = weapon.get_i8("bullet_y")?;
    let bullet_dx = weapon.get_i16("bullet_dx")?;
    let bullet_dy = weapon.get_i16("bullet_dy")?;
    let damage = weapon.get_u16("bullet_damage")?;

    Ok(PlayerCharWeapon {
        name,
        req_collectable_id,
        shoot: PlayerCharWeaponShoot {
            anim_id: shoot_anim_ref.get_asset_id(project_data)?,
            sfx_id: shoot_sfx_ref.get_asset_id_or_none(project_data)?,
            anim_len: shoot_anim_len,
            crouch_x,
            crouch_y,
        },
        bullet: PlayerCharWeaponBullet {
            anim_id: bullet_anim_ref.get_asset_id(project_data)?,
            hit_anim_id: bullet_hit_anim_ref.get_asset_id_or_none(project_data)?,
            hit_sfx_id: bullet_hit_sfx_ref.get_asset_id_or_none(project_data)?,
            ammo_pickup_id,
            initial_delay: bullet_initial_delay,
            interval: bullet_interval,
            x: bullet_x,
            y: bullet_y,
            dx: bullet_dx,
            dy: bullet_dy,
            damage,
        },
    })
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<PlayerChar> {
    let num_suits = asset_struct.get_u16("num_suits")?;
    let suits_array = asset_struct.get_array_ref("suits")?;

    let name = project_data.extract_asset_name("player_char_suits_", suits_array)?;

    let suits_data = suits_array.get_struct_array(project_data)?;
    let suits = suits_data.values.iter().enumerate().map(|(index, suit)| {
        let name = project_data
            .get_asset_data_name(index, "PLAYER_CHAR", name, "SUIT")
            .unwrap_or_else(|| format!("suit_{}", index));
        conv_suit(suit, name, project_data)
    }).collect::<Result<Vec<_>>>()?;

    let num_weapons = asset_struct.get_u16("num_weapons")?;
    let weapons_array = asset_struct.get_array_ref("weapons")?;

    let weapons_data = weapons_array.get_struct_array(project_data)?;
    let weapons = weapons_data.values.iter().enumerate().map(|(index, weapon)| {
        let name = project_data
            .get_asset_data_name(index, "PLAYER_CHAR", name, "WEAPON")
            .unwrap_or_else(|| format!("weapon_{}", index));
        conv_weapon(weapon, name, project_data)
    }).collect::<Result<Vec<_>>>()?;

    if num_suits as usize != suits.len() {
        return error(
            format!("invalid number of player char suits: expected {}, got {}", num_suits, suits.len()),
            suits_array.pos
        );
    }

    if num_weapons as usize != weapons.len() {
        return error(
            format!("invalid number of player char weapons: expected {}, got {}", num_weapons, weapons.len()),
            weapons_array.pos
        );
    }

    Ok(PlayerChar {
        asset: DataAsset::new(DataAssetType::PlayerChar, asset_id, DataAsset::identifier_to_name(name)),
        suits,
        weapons,
    })
}
