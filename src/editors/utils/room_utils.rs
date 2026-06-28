use crate::data_asset::{
    AssetIdCollection,
    RoomTriggerType,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoomTriggerTypeSel {
    Unknown,
    Door,
    Trap,
    PlayerSpawn,
    EnemySpawn,
}

impl RoomTriggerTypeSel {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown { .. } => RoomTriggerTypeSel::Unknown,
            RoomTriggerType::Door { .. } => RoomTriggerTypeSel::Door,
            RoomTriggerType::PlayerSpawn { .. } => RoomTriggerTypeSel::PlayerSpawn,
            RoomTriggerType::EnemySpawn { .. } => RoomTriggerTypeSel::EnemySpawn,
            RoomTriggerType::Trap { .. } => RoomTriggerTypeSel::Trap,
        }
    }

    pub fn convert_trigger_type(&self, trigger_type: &mut RoomTriggerType, asset_ids: &AssetIdCollection) {
        match self {
            RoomTriggerTypeSel::Unknown if ! matches!(trigger_type, RoomTriggerType::Unknown {..}) => {
                *trigger_type = RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0 };
            }
            RoomTriggerTypeSel::Trap if ! matches!(trigger_type, RoomTriggerType::Trap {..}) => {
                *trigger_type = RoomTriggerType::Trap { width: 64, height: 64, type_id: 0 };
            }
            RoomTriggerTypeSel::PlayerSpawn if ! matches!(trigger_type, RoomTriggerType::PlayerSpawn {..}) => {
                *trigger_type = RoomTriggerType::PlayerSpawn { direction: 0 };
            }
            RoomTriggerTypeSel::EnemySpawn if ! matches!(trigger_type, RoomTriggerType::EnemySpawn {..}) => {
                if let Some(animation_id) = asset_ids.animations.get_first() {
                    *trigger_type = RoomTriggerType::EnemySpawn { animation_id };
                }
            }
            RoomTriggerTypeSel::Door if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                if let Some(room_id) = asset_ids.rooms.get_first() {
                    *trigger_type = RoomTriggerType::Door { room_id, door_id: 0 };
                }
            }
            _ => {}
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            RoomTriggerTypeSel::Unknown => "any",
            RoomTriggerTypeSel::Door => "door",
            RoomTriggerTypeSel::Trap => "trap",
            RoomTriggerTypeSel::PlayerSpawn => "player spawn",
            RoomTriggerTypeSel::EnemySpawn => "enemy spawn",
        }
    }
}
