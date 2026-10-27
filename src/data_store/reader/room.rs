use std::io::Result;
use std::collections::HashMap;
use std::sync::LazyLock;

use super::{
    error,
    Value,
    ValueDef,
    ValueDefStruct,
    ValueStruct,
    ProjectData,
    ProjectDataReader,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Room,
    RoomMap,
    RoomTrigger,
    RoomParallax,
    RoomTriggerType,
    RoomTriggerTypeIdent,
    RoomTriggerMapLayer,
    RoomEntityDirection,
};

pub fn get_asset_def() -> ValueDefStruct {
    ValueDefStruct::new(vec![
        (String::from("num_maps"), ValueDef::U16),
        (String::from("num_triggers"), ValueDef::U16),
        (String::from("maps"), ValueDef::ArrayRef),      // ROOM_MAP_INFO
        (String::from("triggers"), ValueDef::ArrayRef),  // ROOM_TRIGGER_INFO
        (String::from("parallax"), ValueDef::Struct(ValueDefStruct::new(vec![
            (String::from("tileset_id"), ValueDef::AssetRef),
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("tiles"), ValueDef::ArrayRef),
        ]))),
    ])
}

pub fn get_global_struct_defs() -> Vec<(String, ValueDefStruct)> {
    vec![
        (String::from("ROOM_MAP_INFO"), ValueDefStruct::new(vec![
            (String::from("x"), ValueDef::U16),
            (String::from("y"), ValueDef::U16),
            (String::from("map"), ValueDef::AssetRef),
        ])),
        (String::from("ROOM_TRIGGER_INFO"), ValueDefStruct::new(vec![
            (String::from("type"), ValueDef::Identifier),
            (String::from("trigger_id"), ValueDef::U16),
            (String::from("x"), ValueDef::I16),
            (String::from("y"), ValueDef::I16),
            (String::from("trigger"), ValueDef::Custom(custom_read_room_trigger_info)),  // ValueStruct
        ])),
    ]
}

// structs inside the room trigger data union
static TRIGGER_VALUE_TYPES: LazyLock<HashMap<String,ValueDefStruct>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("any"), ValueDefStruct::new(vec![
            (String::from("data0"), ValueDef::U32),
            (String::from("data1"), ValueDef::U32),
            (String::from("data2"), ValueDef::U32),
            (String::from("data3"), ValueDef::U32),
            (String::from("data4"), ValueDef::U32),
            (String::from("data5"), ValueDef::U32),
        ])),

        (String::from("door"), ValueDefStruct::new(vec![
            (String::from("dest_room"), ValueDef::AssetRef),
            (String::from("dest_trigger_id"), ValueDef::U16),
            (String::from("req_flag_id"), ValueDef::U16),
        ])),

        (String::from("player_spawn"), ValueDefStruct::new(vec![
            (String::from("direction"), ValueDef::U8),
        ])),

        (String::from("enemy_spawn"), ValueDefStruct::new(vec![
            (String::from("enemy_id"), ValueDef::U16),
            (String::from("enemy_type"), ValueDef::U16),
            (String::from("animation"), ValueDef::AssetRef),
            (String::from("direction"), ValueDef::U8),
            (String::from("req_flag_id"), ValueDef::U16),
        ])),

        (String::from("wall_button"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("set_effect_id"), ValueDef::U16),
            (String::from("set_flag_id"), ValueDef::U16),
            (String::from("req_flag_id"), ValueDef::U16),
            (String::from("req_collectable_id"), ValueDef::U16),
        ])),

        (String::from("detector_button"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("set_effect_id"), ValueDef::U16),
            (String::from("set_flag_id"), ValueDef::U16),
            (String::from("req_flag_id"), ValueDef::U16),
            (String::from("req_collectable_id"), ValueDef::U16),
        ])),

        (String::from("unblock_tiles"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("disable_animation"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("change_tiles"), ValueDefStruct::new(vec![
            (String::from("map_index"), ValueDef::U16),
            (String::from("tile_change_index"), ValueDef::U16),
            (String::from("map_layer"), ValueDef::U8),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("play_sfx"), ValueDefStruct::new(vec![
            (String::from("sfx"), ValueDef::AssetRef),
            (String::from("sound_id"), ValueDef::U16),
            (String::from("volume"), ValueDef::U16),
            (String::from("pitch_shift"), ValueDef::U16),
            (String::from("play_loop"), ValueDef::U8),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("stop_sfx"), ValueDefStruct::new(vec![
            (String::from("sound_id"), ValueDef::U16),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("remove_enemy"), ValueDefStruct::new(vec![
            (String::from("enemy_id"), ValueDef::U16),
            (String::from("on_effect_id"), ValueDef::U16),
        ])),

        (String::from("get_upgrade"), ValueDefStruct::new(vec![
            (String::from("get_upgrade_id"), ValueDef::U16),
        ])),

        (String::from("get_collectable"), ValueDefStruct::new(vec![
            (String::from("get_collectable_id"), ValueDef::U16),
        ])),

        (String::from("get_pickup"), ValueDefStruct::new(vec![
            (String::from("get_pickup_id"), ValueDef::U16),
            (String::from("respawn_delay"), ValueDef::U16),
            (String::from("sfx"), ValueDef::AssetRef),
        ])),
    ])
});

pub fn read_custom_global_struct(reader: &mut ProjectDataReader, struct_tag: &str) -> Result<bool> {
    if struct_tag == "ROOM_SCRIPT" {   // read room script table
        reader.expect_punct('*')?;
        reader.expect_any_ident("room script table identifier")?;
        reader.expect_punct('[')?;
        reader.expect_punct(']')?;
        reader.expect_punct('=')?;
        reader.expect_punct('{')?;
        while let Some(t) = reader.read_loop()? {
            if let Some(ident) = t.get_ident() {
                if ident != "NULL" {
                    return error(format!("expected 'NULL' or '&', found '{}'", ident), t.pos);
                }
                // room has no script; do nothing
                continue;
            }
            if ! t.is_punct('&') {
                return error(format!("expected 'NULL' or '&', found '{}'", t), t.pos);
            }
            let mut ident_token = reader.expect_any_ident("room script table")?;
            if let Some(ident) = ident_token.drain_ident() &&
                let Some(no_prefix) = ident.strip_prefix(&reader.data.prefix_lower) &&
                let Some(no_table_prefix) = no_prefix.strip_prefix("room_script_table_") {
                    reader.data.room_names_with_scripts.insert(no_table_prefix.to_owned());
                }
        }
        reader.expect_punct(';')?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn custom_read_room_trigger_info(reader: &mut ProjectDataReader) -> Result<Value> {
    reader.expect_punct('.')?;
    let mut trigger_type_token = reader.expect_any_ident("trigger union member name")?;
    reader.expect_punct('=')?;
    if let Some(trigger_type) = trigger_type_token.drain_ident() {
        if let Some(struct_def) = TRIGGER_VALUE_TYPES.get(&trigger_type) {
            Ok(Value::Struct(reader.read_struct(struct_def)?))
        } else {
            error(format!("unknown trigger type: {}", trigger_type), trigger_type_token.pos)
        }
    } else {
        error(format!("unexpected {}", trigger_type_token), trigger_type_token.pos)
    }
}

fn conv_map(map: &ValueStruct, project_data: &ProjectData) -> Result<RoomMap> {
    let x = map.get_u16("x")?;
    let y = map.get_u16("y")?;
    let map_ref = map.get_asset_ref("map")?;
    Ok(RoomMap {
        x,
        y,
        map_id: map_ref.get_asset_id(project_data)?,
    })
}

fn conv_trigger_any(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::Unknown {
        data0: data.get_u16("data0")?,
        data1: data.get_u16("data1")?,
        data2: data.get_u16("data2")?,
        data3: data.get_u16("data3")?,
        data4: data.get_u16("data4")?,
        data5: data.get_u16("data5")?,
        data6: data.get_u16("data6")?,
        data7: data.get_u16("data7")?,
    })
}

fn conv_trigger_door(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let dest_room_ref = data.get_asset_ref("dest_room")?;
    let dest_trigger_id = data.get_u16("dest_trigger_id")?;
    let req_flag_id = data.get_u16("req_flag_id")?;
    Ok(RoomTriggerType::Door {
        dest_room_id: dest_room_ref.get_asset_id(project_data)?,
        dest_trigger_id,
        req_flag_id,
    })
}

fn conv_trigger_player_spawn(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    let direction = data.get_u8("direction")?;
    Ok(RoomTriggerType::PlayerSpawn {
        direction: RoomEntityDirection::from(direction),
    })
}

fn conv_trigger_enemy_spawn(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let enemy_id = data.get_u16("enemy_id")?;
    let enemy_type = data.get_u16("enemy_type")?;
    let animation_ref = data.get_asset_ref("animation")?;
    let direction = data.get_u8("direction")?;
    let req_flag_id = data.get_u16("req_flag_id")?;
    Ok(RoomTriggerType::EnemySpawn {
        enemy_id,
        enemy_type: enemy_type.into(),
        animation_id: animation_ref.get_asset_id(project_data)?,
        direction: RoomEntityDirection::from(direction),
        req_flag_id,
    })
}

fn conv_trigger_wall_button(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::WallButton {
        width: data.get_u16("width")?,
        height: data.get_u16("height")?,
        set_effect_id: data.get_u16("set_effect_id")?,
        set_flag_id: data.get_u16("set_flag_id")?,
        req_flag_id: data.get_u16("req_flag_id")?,
        req_collectable_id: data.get_u16("req_collectable_id")?,
    })
}

fn conv_trigger_detector_button(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::DetectorButton {
        width: data.get_u16("width")?,
        height: data.get_u16("height")?,
        set_effect_id: data.get_u16("set_effect_id")?,
        set_flag_id: data.get_u16("set_flag_id")?,
        req_flag_id: data.get_u16("req_flag_id")?,
        req_collectable_id: data.get_u16("req_collectable_id")?,
    })
}

fn conv_trigger_unblock_tiles(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::UnblockTiles {
        width: data.get_u16("width")?,
        height: data.get_u16("height")?,
        on_effect_id: data.get_u16("on_effect_id")?,
    })
}

fn conv_trigger_disable_animation(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::DisableAnimation {
        width: data.get_u16("width")?,
        height: data.get_u16("height")?,
        on_effect_id: data.get_u16("on_effect_id")?,
    })
}

fn conv_trigger_change_tiles(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::ChangeTiles {
        map_index: data.get_u16("map_index")?,
        tile_change_index: data.get_u16("tile_change_index")?,
        map_layer: RoomTriggerMapLayer::from(data.get_u8("map_layer")?),
        on_effect_id: data.get_u16("on_effect_id")?,
    })
}

fn conv_trigger_remove_enemy(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::RemoveEnemy {
        enemy_id: data.get_u16("enemy_id")?,
        on_effect_id: data.get_u16("on_effect_id")?,
    })
}

fn conv_trigger_play_sfx(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let sfx_ref = data.get_asset_ref("sfx")?;
    let sound_id = data.get_u16("sound_id")?;
    let pitch_shift = data.get_u16("pitch_shift")?;
    let volume = data.get_u16("volume")?;
    let play_loop = data.get_u8("play_loop")?;
    let on_effect_id = data.get_u16("on_effect_id")?;
    Ok(RoomTriggerType::PlaySfx {
        sfx_id: sfx_ref.get_asset_id(project_data)?,
        sound_id,
        pitch_shift,
        volume,
        play_loop,
        on_effect_id,
    })
}

fn conv_trigger_stop_sfx(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::StopSfx {
        sound_id: data.get_u16("sound_id")?,
        on_effect_id: data.get_u16("on_effect_id")?,
    })
}

fn conv_trigger_get_upgrade(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::GetUpgrade {
        get_upgrade_id: data.get_u16("get_upgrade_id")?,
    })
}

fn conv_trigger_get_collectable(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::GetCollectable {
        get_collectable_id: data.get_u16("get_collectable_id")?,
    })
}

fn conv_trigger_get_pickup(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let get_pickup_id = data.get_u16("get_pickup_id")?;
    let respawn_delay = data.get_u16("respawn_delay")?;
    let sfx_ref = data.get_asset_ref("sfx")?;
    Ok(RoomTriggerType::GetPickup {
        get_pickup_id,
        respawn_delay,
        sfx_id: sfx_ref.get_asset_id_or_none(project_data)?,
    })
}

fn conv_trigger(trigger: &ValueStruct, name: String, project_data: &ProjectData) -> Result<RoomTrigger> {
    let trigger_type = trigger.get_identifier("type")?;
    let trigger_id = trigger.get_u16("trigger_id")?;
    let x = trigger.get_i16("x")?;
    let y = trigger.get_i16("y")?;
    let type_data = trigger.get_struct("trigger")?;

    let prefix = &project_data.prefix_upper;
    let trigger_type = if RoomTriggerTypeIdent::Unknown.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_any(type_data, project_data)?
    } else if RoomTriggerTypeIdent::Door.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_door(type_data, project_data)?
    } else if RoomTriggerTypeIdent::PlayerSpawn.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_player_spawn(type_data, project_data)?
    } else if RoomTriggerTypeIdent::EnemySpawn.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_enemy_spawn(type_data, project_data)?
    } else if RoomTriggerTypeIdent::WallButton.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_wall_button(type_data, project_data)?
    } else if RoomTriggerTypeIdent::DetectorButton.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_detector_button(type_data, project_data)?
    } else if RoomTriggerTypeIdent::UnblockTiles.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_unblock_tiles(type_data, project_data)?
    } else if RoomTriggerTypeIdent::DisableAnimation.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_disable_animation(type_data, project_data)?
    } else if RoomTriggerTypeIdent::ChangeTiles.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_change_tiles(type_data, project_data)?
    } else if RoomTriggerTypeIdent::RemoveEnemy.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_remove_enemy(type_data, project_data)?
    } else if RoomTriggerTypeIdent::PlaySfx.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_play_sfx(type_data, project_data)?
    } else if RoomTriggerTypeIdent::StopSfx.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_stop_sfx(type_data, project_data)?
    } else if RoomTriggerTypeIdent::GetUpgrade.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_get_upgrade(type_data, project_data)?
    } else if RoomTriggerTypeIdent::GetCollectable.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_get_collectable(type_data, project_data)?
    } else if RoomTriggerTypeIdent::GetPickup.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_get_pickup(type_data, project_data)?
    } else {
        error(format!("unknown trigger type ident: {}", trigger_type.name), trigger_type.pos)?
    };

    Ok(RoomTrigger {
        name_id: name.replace("__", "/"),
        trigger_id,
        x,
        y,
        trigger_type,
    })
}

fn conv_parallax(para_struct: &ValueStruct, project_data: &ProjectData) -> Result<RoomParallax> {
    let tileset_ref = para_struct.get_asset_ref("tileset_id")?;
    let width = para_struct.get_u16("width")?;
    let height = para_struct.get_u16("height")?;
    let tiles_array = para_struct.get_array_ref("tiles")?;
    let tileset_id = tileset_ref.get_asset_id_or_none(project_data)?;
    let width = width as u32;
    let height = height as u32;
    if tileset_id.is_none() && ! tiles_array.is_null() {
        return error("parallax tiles must be NULL when tileset is NULL", tiles_array.pos);
    } else if tileset_id.is_some() && tiles_array.is_null() {
        return error("parallax tiles must no be NULL when tileset is not NULL", tiles_array.pos);
    }
    let tiles = if tiles_array.is_null() {
        Vec::new()
    } else {
        let tiles = tiles_array.get_u8_array(project_data)?;
        let want_size = width as usize * height as usize;
        if tiles.len() != want_size {
            return error(
                format!("invalid parallax tiles size: expected {}, got {}", want_size, tiles.len()),
                tiles_array.pos
            );
        }
        tiles.to_vec()
    };

    Ok(RoomParallax {
        tileset_id,
        width,
        height,
        tiles,
    })
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<Room> {
    let num_maps = asset_struct.get_u16("num_maps")?;
    let num_triggers = asset_struct.get_u16("num_triggers")?;
    let maps_array = asset_struct.get_array_ref("maps")?;
    let triggers_array = asset_struct.get_array_ref("triggers")?;
    let parallax = asset_struct.get_struct("parallax")?;

    let name = project_data.extract_asset_name("room_maps_", maps_array)?;
    let maps = maps_array.get_struct_array(project_data)?;
    let triggers = triggers_array.get_struct_array(project_data)?;

    let maps: Vec<_> = maps.values.iter().map(|m| conv_map(m, project_data)).collect::<Result<Vec<_>>>()?;
    let triggers: Vec<_> = triggers.values.iter().enumerate().map(|(index, trigger)| {
        let name = project_data
            .get_asset_data_name(index, "ROOM", name, "TRG")
            .unwrap_or_else(|| format!("trigger_{}", index));
        conv_trigger(trigger, name, project_data)
    }).collect::<Result<Vec<_>>>()?;

    if num_maps as usize != maps.len() {
        return error(
            format!("invalid number of room maps: expected {}, got {}", num_maps, maps.len()),
            maps_array.pos
        );
    }
    if num_triggers as usize != triggers.len() {
        return error(
            format!("invalid number of room triggers: expected {}, got {}", num_triggers, triggers.len()),
            triggers_array.pos
        );
    }
    let has_script = project_data.room_names_with_scripts.contains(name);


    Ok(Room {
        asset: DataAsset::new(DataAssetType::Room, asset_id, DataAsset::identifier_to_name(name)),
        maps,
        triggers,
        has_script,
        parallax: conv_parallax(parallax, project_data)?,
    })
}
