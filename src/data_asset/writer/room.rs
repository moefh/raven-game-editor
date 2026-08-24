use std::io::{Result, Error};
use std::collections::HashMap;

use super::{
    ProjectDataWriter,
    IdentStore,
};
use super::super::{
    DataAssetId,
    DataAssetType,
    Room,
    RoomTriggerType,
    RoomTriggerTypeIdent,
};

fn write_room_maps(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    writer.write(format!("static const struct {}_ROOM_MAP_INFO {}_room_maps_{}[] = {{\n",
        writer.ident.prefix_upper, writer.ident.prefix_lower, name_id));
    for room_map in room.maps.iter() {
        let map_index = writer.ident.get_asset_index(DataAssetType::MapData, room_map.map_id)?;
        writer.write(format!("  {{ {}, {}, &{}_maps[{}] }},\n", room_map.x, room_map.y, writer.ident.prefix_lower, map_index));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_room_triggers(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    writer.write(format!("static const struct {}_ROOM_TRIGGER_INFO {}_room_triggers_{}[] = {{\n",
        writer.ident.prefix_upper, writer.ident.prefix_lower, name_id));
    for trg in room.triggers.iter() {
        let enum_type_ident = RoomTriggerTypeIdent::from_trigger_type(&trg.trigger_type).enum_ident();

        writer.write("  {  ");
        writer.write(format!("{}_{}, {}, {}, {}, ", writer.ident.prefix_upper, enum_type_ident, trg.trigger_id, trg.x, trg.y));
        match trg.trigger_type {
            RoomTriggerType::Unknown { data0, data1, data2, data3 } => {
                writer.write(format!(".any = {{ {}, {}, {}, {} }}", data0, data1, data2, data3));
            }
            RoomTriggerType::Door { dest_room_id, dest_trigger_id } => {
                let dest_room_index = writer.ident.get_asset_index(DataAssetType::Room, dest_room_id)?;
                writer.write(
                    format!(
                        ".door = {{ &{}_rooms[{}], {} }}",
                        writer.ident.prefix_lower, dest_room_index, dest_trigger_id
                    )
                );
            }
            RoomTriggerType::PlayerSpawn { direction } => {
                writer.write(format!(".player_spawn = {{ {} }}", direction.value()));
            }
            RoomTriggerType::EnemySpawn { animation_id, enemy_type, direction } => {
                let animation_index = writer.ident.get_asset_index(DataAssetType::SpriteAnimation, animation_id)?;
                writer.write(
                    format!(
                        ".enemy_spawn = {{ &{}_sprite_animations[{}], {}, {} }}",
                        writer.ident.prefix_lower, animation_index, enemy_type, direction.value()
                    )
                );
            }
            RoomTriggerType::Trap { width, height, trap_type } => {
                writer.write(format!(".trap = {{ {}, {}, {} }}", width, height, trap_type));
            }
        }
        writer.write(" },\n");
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_rooms(writer: &ProjectDataWriter, room_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === ROOMS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in room_ids.iter() {
        if let Some(room) = writer.store.assets.rooms.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Room, *id)?;
            write_room_maps(writer, room, name_id)?;
            write_room_triggers(writer, room, name_id)?;
        }
    }

    writer.log(format!("-> writing {} rooms", writer.store.asset_ids.rooms.len()));
    writer.write(format!(
        "const struct {}_ROOM {}_rooms[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in room_ids.iter() {
        if let Some(room) = writer.store.assets.rooms.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Room, *id)?;
            writer.write(format!(
                "  {{ {}, {}, {}_room_maps_{}, {}_room_triggers_{} }},\n",
                room.maps.len(),
                room.triggers.len(),
                writer.ident.prefix_lower,
                name_id,
                writer.ident.prefix_lower,
                name_id
            ));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_room_trigger_names(writer: &ProjectDataWriter, room: &Room, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_ROOM_{}_TRG_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, trg) in room.triggers.iter().enumerate() {
        let use_name_id = if trg.name_id == "names" { "names0" } else { &trg.name_id };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let trg_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of trigger {} room {}", index, room.asset.id))
        })?;
        writer.write(format!("  {}_ROOM_{}_TRG_{},\n", writer.ident.prefix_upper, name_id_upper, trg_name_id.to_ascii_uppercase()));
    }
    writer.write(format!("  {}_ROOM_{}_NUM_TRIGGERS,\n", writer.ident.prefix_upper, name_id_upper));
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_room_item_names(writer: &ProjectDataWriter, room_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === ROOM ITEM NAMES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in room_ids.iter() {
        if let Some(room) = writer.store.assets.rooms.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Room, *id)?;
            let name_id_upper = name_id.to_ascii_uppercase();
            write_room_trigger_names(writer, room, &name_id_upper)?;
        }
    }

    Ok(())
}

pub fn write_scripts(writer: &ProjectDataWriter, room_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === ROOM SCRIPTS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    writer.write(format!("#if {}_ADD_ROOM_SCRIPTS\n", writer.ident.prefix_upper));
    writer.write("\n");

    for id in room_ids.iter() {
        if writer.store.assets.rooms.get(id).is_some_and(|room| room.has_script) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Room, *id)?;
            writer.write(format!(
                "extern const struct {}_ROOM_SCRIPT {}_room_script_table_{};\n",
                writer.ident.prefix_upper,
                writer.ident.prefix_lower,
                name_id
            ));
        }
    }

    writer.write("\n");
    writer.write(format!(
        "const struct {}_ROOM_SCRIPT *{}_room_script_table[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in room_ids.iter() {
        if let Some(room) = writer.store.assets.rooms.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Room, room.asset.id)?;
            if room.has_script {
                writer.write(format!("  &{}_room_script_table_{},\n", writer.ident.prefix_lower, name_id));
            } else {
                writer.write(format!("  NULL, // {}\n", room.asset.name));
            }
        }
    }
    writer.write("};\n");

    writer.write("\n");
    writer.write(format!("#endif /* {}_ADD_ROOM_SCRIPTS */\n", writer.ident.prefix_upper));
    writer.write("\n");

    Ok(())
}
