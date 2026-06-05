use std::io::Result;

use super::{
    err,
    error,
    ReaderAssetIndex,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    AssetIdCollection,
    Room,
    RoomMap,
    RoomEntity,
    RoomEntityType,
    RoomTrigger,
    RoomTriggerType,
};

pub struct MapCreationData {
    pub x: u16,
    pub y: u16,
    pub map: ReaderAssetIndex,
}

impl MapCreationData {
    fn into_room_map(self, asset_ids: &AssetIdCollection) -> Result<RoomMap> {
        Ok(RoomMap{
            x: self.x,
            y: self.y,
            map_id: *asset_ids.maps.get(self.map.index).ok_or_else(|| {
                err(format!("invalid map reference: index {}", self.map.index), self.map.pos)
            })?,
        })
    }
}

pub struct EntityCreationData {
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub entity_type: EntityTypeCreationData,
}

impl EntityCreationData {
    fn into_room_entity(self, asset_ids: &AssetIdCollection) -> Result<RoomEntity> {
        Ok(RoomEntity {
            name_id: self.name_id,
            x: self.x,
            y: self.y,
            entity_type: self.entity_type.into_room_entity_type(asset_ids)?,
        })
    }
}

pub enum EntityTypeCreationData {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    Enemy { animation: ReaderAssetIndex },
}

impl EntityTypeCreationData {
    fn into_room_entity_type(self, asset_ids: &AssetIdCollection) -> Result<RoomEntityType> {
        match self {
            EntityTypeCreationData::Unknown { data0, data1, data2, data3 } => {
                Ok(RoomEntityType::Unknown { data0, data1, data2, data3 })
            }
            EntityTypeCreationData::Enemy { animation } => {
                if let Some(&animation_id) = asset_ids.animations.get(animation.index) {
                    Ok(RoomEntityType::Enemy { animation_id })
                } else {
                    error(format!("invalid sprite animation reference: index {}", animation.index),
                          animation.pos)
                }
            }
        }
    }
}

pub struct TriggerCreationData {
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub trigger_type: TriggerTypeCreationData,
}

impl TriggerCreationData {
    fn into_room_trigger(self, asset_ids: &AssetIdCollection) -> Result<RoomTrigger> {
        Ok(RoomTrigger {
            name_id: self.name_id,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            trigger_type: self.trigger_type.into_room_trigger_type(asset_ids)?,
        })
    }
}

pub enum TriggerTypeCreationData {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    Door { room: ReaderAssetIndex, door_id: u16 },
}

impl TriggerTypeCreationData {
    fn into_room_trigger_type(self, asset_ids: &AssetIdCollection) -> Result<RoomTriggerType> {
        match self {
            TriggerTypeCreationData::Unknown { data0, data1, data2, data3 } => {
                Ok(RoomTriggerType::Unknown { data0, data1, data2, data3 })
            }
            TriggerTypeCreationData::Door { room, door_id } => {
                if let Some(&room_id) = asset_ids.rooms.get(room.index) {
                    Ok(RoomTriggerType::Door { room_id, door_id })
                } else {
                    error(format!("invalid room reference: index {}", room.index), room.pos)
                }
            }
        }
    }
}

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub maps: Vec<MapCreationData>,
    pub entities: Vec<EntityCreationData>,
    pub triggers: Vec<TriggerCreationData>,
}

impl CreationData {
    pub fn into_room(self, asset_ids: &AssetIdCollection) -> Result<Room> {
        let maps: Result<Vec<_>> = self.maps.into_iter().map(|m| m.into_room_map(asset_ids)).collect();
        let triggers: Result<Vec<_>> = self.triggers.into_iter().map(|t| t.into_room_trigger(asset_ids)).collect();
        let entities: Result<Vec<_>> = self.entities.into_iter().map(|e| e.into_room_entity(asset_ids)).collect();
        Ok(Room {
            asset: DataAsset::new(DataAssetType::Room, self.asset_id, self.name),
            maps: maps?,
            triggers: triggers?,
            entities: entities?,
        })
    }
}
