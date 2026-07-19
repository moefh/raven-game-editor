use std::collections::{
    HashMap,
    BTreeMap,
};

use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    World,
};

use super::AssetProblem;

fn check_world(world: &World) -> Vec<AssetProblem> {
    let mut problems = Vec::new();

    let mut used_rooms = HashMap::new();
    for (region_index, region) in world.regions.iter().enumerate() {
        for room_id in region.rooms.iter() {
            if let Some(other_region_index) = used_rooms.get(room_id) {
                problems.push(AssetProblem::WorldRegionsUsingSameRoom {
                    room_id: *room_id,
                    region1_index: region_index,
                    region2_index: *other_region_index,
                });
            } else {
                used_rooms.insert(room_id, region_index);
            }
        }
    }

    problems
}

pub fn check_worlds(asset_problems: &mut BTreeMap<DataAssetId, Vec<AssetProblem>>, store: &DataAssetStore) {
    for world in store.assets.worlds.iter() {
        asset_problems.insert(world.asset.id, check_world(world));
    }
}
