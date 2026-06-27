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
    Room,
    RoomMap,
    RoomTrigger,
    RoomTriggerType,
    RoomTriggerTypeIdent,
};

fn conv_map(map: &Value, project_data: &ProjectData, pos: TokenPosition) -> Result<RoomMap> {
    if let Value::Struct(value) = map && let [
        Value::U16(x),
        Value::U16(y),
        Value::AssetRef(map_ref),
    ] = &value[..] {
        Ok(RoomMap {
            x: *x,
            y: *y,
            map_id: map_ref.get_asset_id(project_data)?,
        })
    } else {
        error(format!("bad room map data: {:?}", map), pos)
    }
}

fn conv_trigger_any(data: &Vec<Value>, _project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTriggerType> {
    if let [
        Value::U16(data0),
        Value::U16(data1),
        Value::U16(data2),
        Value::U16(data3),
    ] = &data[..] {
        Ok(RoomTriggerType::Unknown {
            data0: *data0,
            data1: *data1,
            data2: *data2,
            data3: *data3,
        })
    } else {
        error(format!("bad room trigger data for any: {:?}", data), pos)
    }
}

fn conv_trigger_door(data: &Vec<Value>, project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTriggerType> {
    if let [
        Value::AssetRef(room_ref),
        Value::U16(door_id),
    ] = &data[..] {
        Ok(RoomTriggerType::Door {
            room_id: room_ref.get_asset_id(project_data)?,
            door_id: *door_id,
        })
    } else {
        error(format!("bad room trigger data for door: {:?}", data), pos)
    }
}

fn conv_trigger_player_spawn(data: &Vec<Value>, _project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTriggerType> {
    if let [ Value::U8(direction) ] = &data[..] {
        Ok(RoomTriggerType::PlayerSpawn { direction: *direction })
    } else {
        error(format!("bad room trigger data for player_spawn: {:?}", data), pos)
    }
}

fn conv_trigger_enemy_spawn(data: &Vec<Value>, project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTriggerType> {
    if let [
        Value::AssetRef(animation_ref),
    ] = &data[..] {
        Ok(RoomTriggerType::EnemySpawn {
            animation_id: animation_ref.get_asset_id(project_data)?,
        })
    } else {
        error(format!("bad room trigger data for enemy_spawn: {:?}", data), pos)
    }
}

fn conv_trigger_trap(data: &Vec<Value>, _project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTriggerType> {
    if let [
        Value::U16(width),
        Value::U16(height),
        Value::U16(type_id),

    ] = &data[..] {
        Ok(RoomTriggerType::Trap {
            width: *width,
            height: *height,
            type_id: *type_id,
        })
    } else {
        error(format!("bad room trigger data for trap: {:?}", data), pos)
    }
}

fn conv_trigger(trigger: &Value, name: String, project_data: &ProjectData, pos: TokenPosition) -> Result<RoomTrigger> {
    if let Value::Struct(value) = trigger && let [
        Value::Identifier(trigger_type),
        Value::I16(x),
        Value::I16(y),
        Value::Struct(type_data),
    ] = &value[..] {
        let prefix = &project_data.prefix_upper;
        let trigger_type = if RoomTriggerTypeIdent::Unknown.matches_enum_ident(&trigger_type.name, prefix) {
            conv_trigger_any(type_data, project_data, trigger_type.pos)?
        } else if RoomTriggerTypeIdent::Door.matches_enum_ident(&trigger_type.name, prefix) {
            conv_trigger_door(type_data, project_data, trigger_type.pos)?
        } else if RoomTriggerTypeIdent::PlayerSpawn.matches_enum_ident(&trigger_type.name, prefix) {
            conv_trigger_player_spawn(type_data, project_data, trigger_type.pos)?
        } else if RoomTriggerTypeIdent::EnemySpawn.matches_enum_ident(&trigger_type.name, prefix) {
            conv_trigger_enemy_spawn(type_data, project_data, trigger_type.pos)?
        } else if RoomTriggerTypeIdent::Trap.matches_enum_ident(&trigger_type.name, prefix) {
            conv_trigger_trap(type_data, project_data, trigger_type.pos)?
        } else {
            error(format!("unknown trigger type: {}", &trigger_type.name), trigger_type.pos)?
        };
        Ok(RoomTrigger {
            name_id: name,
            x: *x,
            y: *y,
            trigger_type,
        })
    } else {
        error(format!("bad room trigger data: {:?}", trigger), pos)
    }
}

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<Room> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U16(num_maps),
        Value::U16(num_triggers),
        Value::ArrayRef(maps_array),
        Value::ArrayRef(triggers_array),
    ] = &value[..] {
        let name = project_data.extract_asset_name("room_maps_", maps_array)?;
        let maps = maps_array.get_struct_array(project_data)?;
        let triggers = triggers_array.get_struct_array(project_data)?;

        let maps: Vec<_> = maps.iter().map(|m| conv_map(m, project_data, asset_def.pos)).collect::<Result<Vec<_>>>()?;
        let triggers: Vec<_> = triggers.iter().enumerate().map(|(index, trigger)| {
            let name = project_data
                .get_asset_data_name(index, "ROOM", name, "TRG")
                .unwrap_or_else(|| format!("trigger_{}", index));
            conv_trigger(trigger, name, project_data, asset_def.pos)
        }).collect::<Result<Vec<_>>>()?;

        if *num_maps as usize != maps.len() {
            return error(format!("invalid number of room maps: expected {}, got {}",
                                 *num_maps, maps.len()), maps_array.pos);
        }
        if *num_triggers as usize != triggers.len() {
            return error(format!("invalid number of room triggers: expected {}, got {}",
                                 *num_triggers, triggers.len()), triggers_array.pos);
        }

        Ok(Room {
            asset: DataAsset::new(DataAssetType::Room, asset_id, DataAsset::identifier_to_name(name)),
            maps,
            triggers,
        })
    } else {
        error(format!("bad room data: {:?}", asset_def.value), asset_def.pos)
    }
}
