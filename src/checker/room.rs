use std::collections::{
    HashSet,
    BTreeMap,
};

use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    AssetList,
    Room,
    RoomTriggerType,
    MapData,
    Tileset,
};

use super::{
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    AssetProblem,
};

fn check_room_maps(room: &Room, maps: &AssetList<MapData>, problems: &mut Vec<AssetProblem>) {
    for map in &room.maps {
        if let Some(map_data) = maps.get(&map.map_id) {
            let map_x = map.x as u32;
            let map_y = map.y as u32;
            if (map_x + map_data.width) * Tileset::TILE_SIZE > i16::MAX as u32 {
                problems.push(AssetProblem::RoomMapInvalidXLocation {
                    map_id: map.map_id,
                    x: map_x + map_data.width,
                });
            }
            if (map_y + map_data.height) * Tileset::TILE_SIZE > i16::MAX as u32 {
                problems.push(AssetProblem::RoomMapInvalidYLocation {
                    map_id: map.map_id,
                    y: map_y + map_data.height,
                });
            }
        } else {
            problems.push(AssetProblem::RoomInvalidMapId {
                map_id: map.map_id,
            });
        }
    }
}

fn check_room_size(room: &Room, maps: &AssetList<MapData>, problems: &mut Vec<AssetProblem>) {
    let room_size = room.maps.iter().fold((0, 0), |max, room_map| {
        match maps.get(&room_map.map_id) {
            Some(map_data) => { (
                max.0.max(room_map.x as u32 + map_data.width),
                max.1.max(room_map.y as u32 + map_data.height)
            ) }
            None => { max }
        }
    });

    if (room_size.0 * Tileset::TILE_SIZE) < SCREEN_WIDTH || (room_size.1 * Tileset::TILE_SIZE) < SCREEN_HEIGHT {
        problems.push(AssetProblem::RoomTooSmall {
            width: room_size.0,
            height: room_size.1,
        });
    }
}

fn check_room_trigger_ids(room: &Room, problems: &mut Vec<AssetProblem>) {
    let num_triggers = room.triggers.len();
    if num_triggers < 1 {
        return;
    }
    let mut warned = HashSet::new();
    for trigger1_index in 0..num_triggers-1 {
        if warned.contains(&trigger1_index) {
            continue;
        }
        let t1 = &room.triggers[trigger1_index];
        for trigger2_index in trigger1_index+1..num_triggers {
            let t2 = &room.triggers[trigger2_index];
            if t1.trigger_id == t2.trigger_id {
                warned.insert(trigger2_index);
                problems.push(AssetProblem::RoomTriggersWithSameId {
                    trigger1_index,
                    trigger2_index,
                    trigger_id: t1.trigger_id,
                });
            }
        }
    }
}

fn is_valid_door(room_id: DataAssetId, trigger_id: u16, rooms: &AssetList<Room>) -> bool {
    if let Some(room) = rooms.get(&room_id) &&
        let Some(trigger) = room.triggers.iter().find(|tr| tr.trigger_id == trigger_id) &&
        matches!(trigger.trigger_type, RoomTriggerType::Door {..}) {
            true
        } else {
            false
        }
}

fn check_room_door_destinations(room: &Room, rooms: &AssetList<Room>, problems: &mut Vec<AssetProblem>) {
    for (trigger_index, trigger) in room.triggers.iter().enumerate() {
        if let RoomTriggerType::Door { dest_room_id, dest_trigger_id } = trigger.trigger_type &&
            ! is_valid_door(dest_room_id, dest_trigger_id, rooms) {
                problems.push(AssetProblem::RoomDoorWithInvalidDestination { trigger_index });
            }
    }
}

fn check_room(room: &Room, rooms: &AssetList<Room>, maps: &AssetList<MapData>) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if room.maps.is_empty() {
        problems.push(AssetProblem::RoomWithNoMaps);
    }

    check_room_maps(room, maps, &mut problems);
    check_room_size(room, maps, &mut problems);
    check_room_trigger_ids(room, &mut problems);
    check_room_door_destinations(room, rooms, &mut problems);

    problems
}

pub fn check_rooms(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for room in store.assets.rooms.iter() {
        asset_problems.insert(room.asset.id, check_room(room, &store.assets.rooms, &store.assets.maps));
    }
}
