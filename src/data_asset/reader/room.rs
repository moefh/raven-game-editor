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
    RoomTriggerType,
    RoomTriggerTypeIdent,
};

pub fn get_asset_def() -> ValueDefStruct {
    ValueDefStruct::new(vec![
        (String::from("num_maps"), ValueDef::U16),
        (String::from("num_triggers"), ValueDef::U16),
        (String::from("maps"), ValueDef::ArrayRef),      // ROOM_MAP_INFO
        (String::from("triggers"), ValueDef::ArrayRef),  // ROOM_TRIGGER_INFO
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
        ])),

        (String::from("door"), ValueDefStruct::new(vec![
            (String::from("dest_room"), ValueDef::AssetRef),
            (String::from("dest_trigger_id"), ValueDef::U16),
        ])),

        (String::from("player_spawn"), ValueDefStruct::new(vec![
            (String::from("direction"), ValueDef::U8),
        ])),

        (String::from("enemy_spawn"), ValueDefStruct::new(vec![
            (String::from("animation"), ValueDef::AssetRef),
        ])),

        (String::from("trap"), ValueDefStruct::new(vec![
            (String::from("width"), ValueDef::U16),
            (String::from("height"), ValueDef::U16),
            (String::from("trap_type"), ValueDef::U16),
        ])),
    ])
});

pub fn read_custom_global_struct(reader: &mut ProjectDataReader, struct_tag: &str) -> Result<bool> {
    if struct_tag == "ROOM_SCRIPT" {   // ignore room script table
        reader.expect_punct('*')?;
        reader.expect_any_ident("room script table identifier")?;
        reader.expect_punct('[')?;
        reader.expect_punct(']')?;
        reader.expect_punct('=')?;
        reader.expect_punct('{')?;
        while let Some(t) = reader.read_loop()? {
            if ! t.is_punct('&') {
                return error(format!("expected '&', found '{}'", t), t.pos);
            }
            reader.expect_any_ident("room script table")?;
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
    })
}

fn conv_trigger_door(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let dest_room_ref = data.get_asset_ref("dest_room")?;
    let dest_trigger_id = data.get_u16("dest_trigger_id")?;
    Ok(RoomTriggerType::Door {
        dest_room_id: dest_room_ref.get_asset_id(project_data)?,
        dest_trigger_id,
    })
}

fn conv_trigger_player_spawn(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::PlayerSpawn {
        direction: data.get_u8("direction")?
    })
}

fn conv_trigger_enemy_spawn(data: &ValueStruct, project_data: &ProjectData) -> Result<RoomTriggerType> {
    let animation_ref = data.get_asset_ref("animation")?;
    Ok(RoomTriggerType::EnemySpawn {
        animation_id: animation_ref.get_asset_id(project_data)?,
    })
}

fn conv_trigger_trap(data: &ValueStruct, _project_data: &ProjectData) -> Result<RoomTriggerType> {
    Ok(RoomTriggerType::Trap {
        width: data.get_u16("width")?,
        height: data.get_u16("height")?,
        trap_type: data.get_u16("trap_type")?,
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
    } else if RoomTriggerTypeIdent::Trap.matches_enum_ident(&trigger_type.name, prefix) {
        conv_trigger_trap(type_data, project_data)?
    } else {
        error(format!("unknown trigger type: {}", trigger_type.name), trigger_type.pos)?
    };

    Ok(RoomTrigger {
        name_id: name,
        trigger_id,
        x,
        y,
        trigger_type,
    })
}

pub fn create(asset_id: DataAssetId, asset_struct: &ValueStruct, project_data: &ProjectData) -> Result<Room> {
    let num_maps = asset_struct.get_u16("num_maps")?;
    let num_triggers = asset_struct.get_u16("num_triggers")?;
    let maps_array = asset_struct.get_array_ref("maps")?;
    let triggers_array = asset_struct.get_array_ref("triggers")?;

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

    Ok(Room {
        asset: DataAsset::new(DataAssetType::Room, asset_id, DataAsset::identifier_to_name(name)),
        maps,
        triggers,
    })
}
