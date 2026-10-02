use crate::data_asset::{
    AssetIdCollection,
    RoomEnemyType,
    RoomTriggerType,
    RoomEntityDirection,
};

#[derive(Clone, Copy, PartialEq)]
pub enum RoomEnemyTypeSel {
    Walker,
    Chiller,
    Hopper,
    Floater,
    Other,
}

impl RoomEnemyTypeSel {
    pub fn text(&self) -> &'static str {
        match self {
            RoomEnemyTypeSel::Walker  => { "walker" }
            RoomEnemyTypeSel::Chiller => { "chiller" }
            RoomEnemyTypeSel::Hopper  => { "hopper" }
            RoomEnemyTypeSel::Floater => { "floater" }
            RoomEnemyTypeSel::Other   => { "other..." }
        }
    }
}

impl From<RoomEnemyType> for RoomEnemyTypeSel {
    fn from(value: RoomEnemyType) -> Self {
        match value {
            RoomEnemyType::Walker   => { RoomEnemyTypeSel::Walker }
            RoomEnemyType::Chiller  => { RoomEnemyTypeSel::Chiller }
            RoomEnemyType::Hopper   => { RoomEnemyTypeSel::Hopper }
            RoomEnemyType::Floater  => { RoomEnemyTypeSel::Floater }
            RoomEnemyType::Other(_) => { RoomEnemyTypeSel::Other }
        }
    }
}

impl From<RoomEnemyTypeSel> for RoomEnemyType {
    fn from(value: RoomEnemyTypeSel) -> Self {
        match value {
            RoomEnemyTypeSel::Walker   => { RoomEnemyType::Walker }
            RoomEnemyTypeSel::Chiller  => { RoomEnemyType::Chiller }
            RoomEnemyTypeSel::Hopper   => { RoomEnemyType::Hopper }
            RoomEnemyTypeSel::Floater  => { RoomEnemyType::Floater }
            RoomEnemyTypeSel::Other    => { RoomEnemyType::Other(4) }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoomTriggerTypeSel {
    Unknown,
    Door,
    Trap,
    PlayerSpawn,
    EnemySpawn,
    WallButton,
    FloorButton,
    UnblockEffect,
    DisableAnimationEffect,
    GetUpgrade,
    GetCollectable,
    GetPickup,
}

impl RoomTriggerTypeSel {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown { .. } => RoomTriggerTypeSel::Unknown,
            RoomTriggerType::Door { .. } => RoomTriggerTypeSel::Door,
            RoomTriggerType::PlayerSpawn { .. } => RoomTriggerTypeSel::PlayerSpawn,
            RoomTriggerType::EnemySpawn { .. } => RoomTriggerTypeSel::EnemySpawn,
            RoomTriggerType::Trap { .. } => RoomTriggerTypeSel::Trap,
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeSel::WallButton }
            RoomTriggerType::FloorButton {..} => { RoomTriggerTypeSel::FloorButton }
            RoomTriggerType::UnblockEffect {..} => { RoomTriggerTypeSel::UnblockEffect }
            RoomTriggerType::DisableAnimationEffect {..} => { RoomTriggerTypeSel::DisableAnimationEffect }
            RoomTriggerType::GetUpgrade {..} => { RoomTriggerTypeSel::GetUpgrade }
            RoomTriggerType::GetCollectable {..} => { RoomTriggerTypeSel::GetCollectable }
            RoomTriggerType::GetPickup {..} => { RoomTriggerTypeSel::GetPickup }
        }
    }

    pub fn convert_trigger_type(&self, trigger_type: &mut RoomTriggerType, asset_ids: &AssetIdCollection) -> bool {
        match self {
            RoomTriggerTypeSel::Unknown if ! matches!(trigger_type, RoomTriggerType::Unknown {..}) => {
                *trigger_type = RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0 };
                true
            }
            RoomTriggerTypeSel::Trap if ! matches!(trigger_type, RoomTriggerType::Trap {..}) => {
                *trigger_type = RoomTriggerType::Trap { width: 64, height: 64, trap_type: 0 };
                true
            }
            RoomTriggerTypeSel::PlayerSpawn if ! matches!(trigger_type, RoomTriggerType::PlayerSpawn {..}) => {
                *trigger_type = RoomTriggerType::PlayerSpawn { direction: RoomEntityDirection::Right };
                true
            }
            RoomTriggerTypeSel::EnemySpawn if ! matches!(trigger_type, RoomTriggerType::EnemySpawn {..}) => {
                if let Some(animation_id) = asset_ids.animations.get_first() {
                    *trigger_type = RoomTriggerType::EnemySpawn {
                        animation_id,
                        enemy_type: RoomEnemyType::Walker,
                        direction: RoomEntityDirection::Right,
                    };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::Door if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                if let Some(room_id) = asset_ids.rooms.get_first() {
                    *trigger_type = RoomTriggerType::Door { dest_room_id: room_id, dest_trigger_id: 0 };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::WallButton if ! matches!(trigger_type, RoomTriggerType::WallButton {..}) => {
                *trigger_type = RoomTriggerType::WallButton {
                    width: 16,
                    height: 16,
                    effect_id: u16::MAX,
                    flag_id: u16::MAX,
                    required_collectable_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::FloorButton if ! matches!(trigger_type, RoomTriggerType::FloorButton {..}) => {
                *trigger_type = RoomTriggerType::FloorButton {
                    width: 16,
                    height: 16,
                    effect_id: u16::MAX,
                    flag_id: u16::MAX,
                };
                true
            }
            RoomTriggerTypeSel::UnblockEffect if ! matches!(trigger_type, RoomTriggerType::UnblockEffect {..}) => {
                *trigger_type = RoomTriggerType::UnblockEffect {
                    width: 64,
                    height: 64,
                    effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::DisableAnimationEffect if ! matches!(trigger_type, RoomTriggerType::DisableAnimationEffect {..}) => {
                *trigger_type = RoomTriggerType::DisableAnimationEffect {
                    width: 64,
                    height: 64,
                    effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetUpgrade if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                *trigger_type = RoomTriggerType::GetUpgrade {
                    upgrade_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetCollectable if ! matches!(trigger_type, RoomTriggerType::GetCollectable {..}) => {
                *trigger_type = RoomTriggerType::GetCollectable {
                    collectable_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetPickup if ! matches!(trigger_type, RoomTriggerType::GetPickup {..}) => {
                *trigger_type = RoomTriggerType::GetPickup {
                    pickup_id: u16::MAX
                };
                true
            }
            _ => { false }
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            RoomTriggerTypeSel::Unknown => "any",
            RoomTriggerTypeSel::Door => "door",
            RoomTriggerTypeSel::Trap => "trap",
            RoomTriggerTypeSel::PlayerSpawn => "player spawn",
            RoomTriggerTypeSel::EnemySpawn => "enemy spawn",
            RoomTriggerTypeSel::WallButton => { "wall button" }
            RoomTriggerTypeSel::FloorButton => { "floor button" }
            RoomTriggerTypeSel::UnblockEffect => { "unblock" }
            RoomTriggerTypeSel::DisableAnimationEffect => { "disable animation" }
            RoomTriggerTypeSel::GetUpgrade => { "upgrade" }
            RoomTriggerTypeSel::GetCollectable => { "collectible" }
            RoomTriggerTypeSel::GetPickup => { "pickup" }
        }
    }
}
