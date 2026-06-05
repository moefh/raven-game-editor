use std::io::Result;

use super::ReaderAssetReference;
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    AssetIdCollection,
    Room,
    RoomMap,
    RoomTrigger,
    RoomTriggerType,
};

pub struct MapCreationData {
    pub x: u16,
    pub y: u16,
    pub map_ref: ReaderAssetReference,
}

impl MapCreationData {
    fn into_room_map(self, asset_ids: &AssetIdCollection) -> Result<RoomMap> {
        Ok(RoomMap{
            x: self.x,
            y: self.y,
            map_id: self.map_ref.get_asset_id(&asset_ids.maps)?,
        })
    }
}

pub struct TriggerCreationData {
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub trigger_type: TriggerTypeCreationData,
}

impl TriggerCreationData {
    fn into_room_trigger(self, asset_ids: &AssetIdCollection) -> Result<RoomTrigger> {
        Ok(RoomTrigger {
            name_id: self.name_id,
            x: self.x,
            y: self.y,
            trigger_type: self.trigger_type.into_room_trigger_type(asset_ids)?,
        })
    }
}

pub enum TriggerTypeCreationData {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    PlayerSpawn { direction: u8 },
    EnemySpawn { animation_ref: ReaderAssetReference },
    Door { room_ref: ReaderAssetReference, door_id: u16 },
    Trap { width: u16, height: u16, type_id: u16 },
}

impl TriggerTypeCreationData {
    pub fn get_enum_ident(&self) -> super::super::RoomTriggerTypeIdent {
        match self {
            TriggerTypeCreationData::Unknown {..} => { super::super::RoomTriggerTypeIdent::Unknown }
            TriggerTypeCreationData::Door {..} => { super::super::RoomTriggerTypeIdent::Door }
            TriggerTypeCreationData::PlayerSpawn {..} => { super::super::RoomTriggerTypeIdent::PlayerSpawn}
            TriggerTypeCreationData::EnemySpawn {..} => { super::super::RoomTriggerTypeIdent::EnemySpawn }
            TriggerTypeCreationData::Trap {..} => { super::super::RoomTriggerTypeIdent::Trap }
        }
    }

    fn into_room_trigger_type(self, asset_ids: &AssetIdCollection) -> Result<RoomTriggerType> {
        match self {
            TriggerTypeCreationData::Unknown { data0, data1, data2, data3 } => {
                Ok(RoomTriggerType::Unknown { data0, data1, data2, data3 })
            }
            TriggerTypeCreationData::Door { room_ref, door_id } => {
                Ok(RoomTriggerType::Door {
                    room_id: room_ref.get_asset_id(&asset_ids.rooms)?,
                    door_id,
                })
            }
            TriggerTypeCreationData::PlayerSpawn { direction } => {
                Ok(RoomTriggerType::PlayerSpawn { direction })
            }
            TriggerTypeCreationData::EnemySpawn { animation_ref } => {
                Ok(RoomTriggerType::EnemySpawn {
                    animation_id: animation_ref.get_asset_id(&asset_ids.animations)?,
                })
            }
            TriggerTypeCreationData::Trap { width, height, type_id } => {
                Ok(RoomTriggerType::Trap {
                    width,
                    height,
                    type_id,
                })
            }
        }
    }
}

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub maps: Vec<MapCreationData>,
    pub triggers: Vec<TriggerCreationData>,
}

impl CreationData {
    pub fn into_room(self, asset_ids: &AssetIdCollection) -> Result<Room> {
        let maps: Result<Vec<_>> = self.maps.into_iter().map(|m| m.into_room_map(asset_ids)).collect();
        let triggers: Result<Vec<_>> = self.triggers.into_iter().map(|t| t.into_room_trigger(asset_ids)).collect();
        Ok(Room {
            asset: DataAsset::new(DataAssetType::Room, self.asset_id, self.name),
            maps: maps?,
            triggers: triggers?,
        })
    }
}
