use std::collections::BTreeMap;

use crate::data_asset::{DataAssetId, DataAssetStore, AssetList, Room, MapData, Tileset};

use super::AssetProblem;

fn check_room(room: &Room, maps: &AssetList<MapData>) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    if room.maps.is_empty() {
        problems.push(AssetProblem::RoomWithNoMaps);
    }

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

    problems
}

pub fn check_rooms(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for room in store.assets.rooms.iter() {
        asset_problems.insert(room.asset.id, check_room(room, &store.assets.maps));
    }
}
