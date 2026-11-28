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
    RoomCutsceneActionType,
    RoomCutsceneActionTypeIdent,
};

fn write_room_maps(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    writer.write(format!("static const struct {}_ROOM_MAP_INFO {}_room_maps_{}[] = {{\n",
        writer.ident.prefix_upper, writer.ident.prefix_lower, name_id));
    for room_map in room.maps.iter() {
        let map_ident = writer.ident.get_asset_ident(DataAssetType::MapData, room_map.map_id)?;
        writer.write(format!("  {{ {}, {}, {} }},\n", room_map.x, room_map.y, map_ident));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_room_triggers(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    writer.write(format!(
        "static const struct {}_ROOM_TRIGGER_INFO {}_room_triggers_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for trg in room.triggers.iter() {
        let enum_type_ident = RoomTriggerTypeIdent::from_trigger_type(&trg.trigger_type).enum_ident();

        writer.write("  {\n");
        writer.write(format!("    {}_{}, {}, {}, {},\n", writer.ident.prefix_upper, enum_type_ident, trg.trigger_id, trg.x, trg.y));
        match trg.trigger_type {
            RoomTriggerType::Unknown { data0, data1, data2, data3, data4, data5, data6, data7 } => {
                writer.write(format!(
                    "    .any = {{ {}, {}, {}, {}, {}, {}, {}, {} }}\n",
                    data0,
                    data1,
                    data2,
                    data3,
                    data4,
                    data5,
                    data6,
                    data7,
                ));
            }
            RoomTriggerType::Door { dest_room_id, dest_trigger_id, req_flag_id } => {
                let dest_room_ident = writer.ident.get_asset_ident(DataAssetType::Room, dest_room_id)?;
                writer.write(
                    format!(
                        "    .door = {{ {}, {}, {} }}\n",
                        dest_room_ident,
                        dest_trigger_id,
                        req_flag_id
                    )
                );
            }
            RoomTriggerType::PlayerSpawn { direction } => {
                writer.write(format!("    .player_spawn = {{ {} }}\n", direction.value()));
            }
            RoomTriggerType::EnemySpawn { enemy_id, enemy_num, direction, req_flag_id } => {
                let enemy_ident = writer.ident.get_asset_ident(DataAssetType::Enemy, enemy_id)?;
                writer.write(
                    format!(
                        "    .enemy_spawn = {{ {}, {}, {}, {} }}\n",
                        enemy_num,
                        enemy_ident,
                        direction.value(),
                        req_flag_id,
                    )
                );
            }
            RoomTriggerType::WallButton {
                width,
                height,
                interact_prompt_id,
                set_effect_id,
                set_flag_id,
                req_flag_id,
                req_collectable_id,
            } => {
                writer.write(format!(
                    "    .wall_button = {{ {}, {}, {}, {}, {}, {}, {} }}\n",
                    width,
                    height,
                    interact_prompt_id,
                    set_effect_id,
                    set_flag_id,
                    req_flag_id,
                    req_collectable_id
                ));
            }
            RoomTriggerType::DetectorButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                writer.write(format!(
                    "    .detector_button = {{ {}, {}, {}, {}, {}, {} }}\n",
                    width,
                    height,
                    set_effect_id,
                    set_flag_id,
                    req_flag_id,
                    req_collectable_id
                ));
            }
            RoomTriggerType::UnblockTiles { width, height, on_effect_id } => {
                writer.write(format!("    .unblock_tiles = {{ {}, {}, {} }}\n", width, height, on_effect_id));
            }
            RoomTriggerType::DisableAnimation { width, height, on_effect_id } => {
                writer.write(format!("    .disable_animation = {{ {}, {}, {} }}\n", width, height, on_effect_id));
            }
            RoomTriggerType::ChangeTiles { map_index, tile_change_index, map_layer, on_effect_id } => {
                writer.write(format!(
                    "    .change_tiles = {{ {}, {}, {}, {} }}\n",
                    map_index,
                    tile_change_index,
                    map_layer.value(),
                    on_effect_id
                ));
            }
            RoomTriggerType::RemoveEnemy { enemy_num, on_effect_id } => {
                writer.write(format!("    .remove_enemy = {{ {}, {} }}\n", enemy_num, on_effect_id));
            }
            RoomTriggerType::PlayAnimation { anim_id, anim_num, direction, layer, on_effect_id, set_effect_id, set_flag_id } => {
                let anim_ident = writer.ident.get_asset_ident(DataAssetType::SpriteAnimation, anim_id)?;
                writer.write(format!(
                    "    .play_animation = {{ {}, {}, {}, {}, {}, {}, {} }}\n",
                    anim_ident,
                    anim_num,
                    direction.value(),
                    layer.value(),
                    on_effect_id,
                    set_effect_id,
                    set_flag_id,
                ));
            }
            RoomTriggerType::StopAnimation { anim_num, on_effect_id } => {
                writer.write(format!(
                    "    .stop_animation = {{ {}, {} }}\n",
                    anim_num,
                    on_effect_id,
                ));
            }
            RoomTriggerType::PlayCutscene { cutscene_index, on_effect_id, req_flag_id, deny_flag_id } => {
                writer.write(format!(
                    "    .play_cutscene = {{ {}, {}, {}, {} }}\n",
                    cutscene_index,
                    on_effect_id,
                    req_flag_id,
                    deny_flag_id,
                ));
            }
            RoomTriggerType::PlaySfx { sfx_id, sound_id, volume, pitch_shift, play_loop, on_effect_id } => {
                let sfx_ident = writer.ident.get_asset_ident(DataAssetType::Sfx, sfx_id)?;
                writer.write(format!(
                    "    .play_sfx = {{ {}, {}, {}, {}, {}, {} }}\n",
                    sfx_ident,
                    sound_id,
                    volume,
                    pitch_shift,
                    play_loop,
                    on_effect_id,
                ));
            }
            RoomTriggerType::StopSfx { sound_id, on_effect_id } => {
                writer.write(format!("    .stop_sfx = {{ {}, {} }}\n", sound_id, on_effect_id));
            }
            RoomTriggerType::GetUpgrade { get_upgrade_id, req_flag_id } => {
                writer.write(format!("    .get_upgrade = {{ {}, {} }}\n", get_upgrade_id, req_flag_id));
            }
            RoomTriggerType::GetCollectable { get_collectable_id, interact_prompt_id, req_flag_id } => {
                writer.write(format!(
                    "    .get_collectable = {{ {}, {}, {} }}\n",
                    get_collectable_id,
                    interact_prompt_id,
                    req_flag_id
                ));
            }
            RoomTriggerType::GetPickup { get_pickup_id, respawn_delay, sfx_id } => {
                let sfx_ident = writer.ident.get_asset_ident_or_none(DataAssetType::Sfx, sfx_id)?;
                writer.write(format!(
                    "    .get_pickup = {{ {}, {}, {}, }}\n",
                    get_pickup_id,
                    respawn_delay,
                    sfx_ident,
                ));
            }
            RoomTriggerType::OnLoadRoom { set_effect_id, set_flag_id, req_flag_id, deny_flag_id } => {
                writer.write(format!(
                    "    .on_load_room = {{ {}, {}, {}, {} }}\n",
                    set_effect_id,
                    set_flag_id,
                    req_flag_id,
                    deny_flag_id,
                ));
            }
        }
        writer.write("  },\n");
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_room_cutscenes(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    for (index, cutscene) in room.cutscenes.iter().enumerate() {
        writer.write(format!(
            "static const struct {}_ROOM_CUTSCENE_ACTION {}_room_cutscene_{}_{}[] = {{\n",
            writer.ident.prefix_upper,
            writer.ident.prefix_lower,
            name_id,
            index,
        ));
        for action in &cutscene.actions {
            let enum_type_ident = RoomCutsceneActionTypeIdent::from_action_type(&action.action_type).enum_ident();

            writer.write("  {\n");
            writer.write(format!(
                "    {}_{},\n",
                writer.ident.prefix_upper,
                enum_type_ident
            ));
            match action.action_type {
                RoomCutsceneActionType::Unknown { data0, data1, data2, data3, data4, data5 } => {
                    writer.write(format!(
                        "    .any = {{ {}, {}, {}, {}, {}, {} }}\n",
                        data0,
                        data1,
                        data2,
                        data3,
                        data4,
                        data5,
                    ));
                }
                RoomCutsceneActionType::WaitFrames { num_frames } => {
                    writer.write(format!(
                        "    .wait_frames = {{ {} }}\n",
                        num_frames,
                    ));
                }
                RoomCutsceneActionType::WaitPlayerAnimationEnd {} => {
                    writer.write(
                        "    .wait_player_anim_end = {}\n",
                    );
                }
                RoomCutsceneActionType::DispelDarkness { speed } => {
                    writer.write(format!(
                        "    .dispel_darkness = {{ {} }}\n",
                        speed,
                    ));
                }
                RoomCutsceneActionType::SetPlayerDarkness { radius, fall_off, off_x, off_y } => {
                    writer.write(format!(
                        "    .set_player_darkness = {{ {}, {}, {}, {} }}\n",
                        radius,
                        fall_off,
                        off_x,
                        off_y,
                    ));
                }
                RoomCutsceneActionType::MovePlayerDarkness { off_x, off_y, speed } => {
                    writer.write(format!(
                        "    .move_player_darkness = {{ {}, {}, {} }}\n",
                        off_x,
                        off_y,
                        speed,
                    ));
                }
                RoomCutsceneActionType::SetPlayerState { state } => {
                    writer.write(format!(
                        "    .set_player_state = {{ {} }}\n",
                        state,
                    ));
                }
                RoomCutsceneActionType::ActivateEffect { set_effect_id } => {
                    writer.write(format!(
                        "    .activate_effect = {{ {} }}\n",
                        set_effect_id,
                    ));
                }
                RoomCutsceneActionType::SetHudEnabled { enabled } => {
                    writer.write(format!(
                        "    .set_hud_enabled = {{ {} }}\n",
                        enabled,
                    ));
                }
            }
            writer.write("  },\n");
        }
        writer.write("};\n");
        writer.write("\n");
    }

    writer.write(format!(
        "static const struct {}_ROOM_CUTSCENE {}_room_cutscenes_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for (index, cutscene) in room.cutscenes.iter().enumerate() {
        writer.write(format!(
            "  {{ {}, {}_room_cutscene_{}_{} }},\n",
            cutscene.actions.len(),
            writer.ident.prefix_lower,
            name_id,
            index,
        ));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_room_parallax_tiles(writer: &ProjectDataWriter, room: &Room, name_id: &str) -> Result<()> {
    if room.parallax.tileset_id.is_none() { return Ok(()); }

    writer.write(format!(
        "static const uint8_t {}_room_parallax_tiles_{}[] = {{",
        writer.ident.prefix_lower,
        name_id
    ));
    for (index, tile) in room.parallax.tiles.iter().enumerate() {
        if index.is_multiple_of(16) { writer.write("\n  "); }
        writer.write(format!("{:#04x},", tile))
    }
    writer.write("\n};\n");
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
            write_room_cutscenes(writer, room, name_id)?;
            write_room_parallax_tiles(writer, room, name_id)?;
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
            writer.write("  {\n");
            writer.write(format!(
                "    {}, {}, {}, {}_room_maps_{}, {}_room_triggers_{}, {}_room_cutscenes_{},\n",
                room.maps.len(),
                room.triggers.len(),
                room.cutscenes.len(),
                writer.ident.prefix_lower,
                name_id,
                writer.ident.prefix_lower,
                name_id,
                writer.ident.prefix_lower,
                name_id,
            ));
            let tileset_ident = writer.ident.get_asset_ident_or_none(DataAssetType::Tileset, room.parallax.tileset_id)?;
            if room.parallax.tileset_id.is_some() && room.parallax.width != 0 && room.parallax.height != 0 {
                writer.write(format!(
                    "    {{ {}, {}, {}, {}_room_parallax_tiles_{} }}\n",
                    tileset_ident,
                    room.parallax.width,
                    room.parallax.height,
                    writer.ident.prefix_lower,
                    name_id,
                ));
            } else {
                writer.write(format!("    {{ {}, 0, 0, NULL }}\n", tileset_ident));
            }
            writer.write("  },\n");
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
        let use_name_id = if trg.name_id == "names" { "names0" } else { &trg.name_id.replace("/", "__") };
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

fn write_room_cutscene_names(writer: &ProjectDataWriter, room: &Room, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_ROOM_{}_CUTSCENE_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, cutscene) in room.cutscenes.iter().enumerate() {
        let use_name_id = if cutscene.name_id == "names" { "names0" } else { &cutscene.name_id };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let cutscene_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of cutscene {} room {}", index, room.asset.id))
        })?;
        writer.write(format!(
            "  {}_ROOM_{}_CUTSCENE_{},\n",
            writer.ident.prefix_upper,
            name_id_upper,
            cutscene_name_id.to_ascii_uppercase()
        ));
    }
    writer.write(format!("  {}_ROOM_{}_NUM_CUTSCENES,\n", writer.ident.prefix_upper, name_id_upper));
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
            write_room_cutscene_names(writer, room, &name_id_upper)?;
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
