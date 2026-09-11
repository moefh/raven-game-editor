use std::collections::{
    HashMap,
    BTreeMap,
};

use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
    World,
};

use super::{
    AssetError,
    AssetWarning,
};

fn check_world(world: &World) -> (Vec<AssetError>, Vec<AssetWarning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if world.regions.is_empty() {
        warnings.push(AssetWarning::WorldWithNoRegions);
    }

    let mut used_rooms = HashMap::new();
    for (region_index, region) in world.regions.iter().enumerate() {
        if region.rooms.is_empty() {
            warnings.push(AssetWarning::WorldRegionWithNoMaps { region_index });
        }
        for room_id in region.rooms.iter() {
            if let Some(other_region_index) = used_rooms.get(room_id) {
                errors.push(AssetError::WorldRegionsUsingSameRoom {
                    room_id: *room_id,
                    region1_index: region_index,
                    region2_index: *other_region_index,
                });
            } else {
                used_rooms.insert(room_id, region_index);
            }
        }
    }

    (errors, warnings)
}

pub fn check_worlds(
    asset_errors: &mut BTreeMap<DataAssetId, Vec<AssetError>>,
    asset_warnings: &mut BTreeMap<DataAssetId, Vec<AssetWarning>>,
    store: &DataAssetStore
) {
    for world in store.assets.worlds.iter() {
        let (errors, warnings) = check_world(world);
        asset_errors.insert(world.asset.id, errors);
        asset_warnings.insert(world.asset.id, warnings);
    }
}
