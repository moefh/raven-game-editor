use super::DataAssetId;

#[derive(Copy, Clone, PartialEq, std::hash::Hash)]
pub enum RoomEntityDirection {
    Right,
    Left,
}

#[derive(Copy, Clone, PartialEq, std::hash::Hash)]
pub enum RoomEnemyType {
    Walker,
    Chiller,
    Hopper,
    Floater,
    Other(u16),
}

impl RoomEnemyType {
    pub fn value(self) -> u16 {
        match self {
            RoomEnemyType::Walker   => { 0 }
            RoomEnemyType::Chiller  => { 1 }
            RoomEnemyType::Hopper   => { 2 }
            RoomEnemyType::Floater  => { 3 }
            RoomEnemyType::Other(n) => { n }
        }
    }
}

impl From<u16> for RoomEnemyType {
    fn from(value: u16) -> Self {
        match value {
            0 => { RoomEnemyType::Walker }
            1 => { RoomEnemyType::Chiller }
            2 => { RoomEnemyType::Hopper }
            3 => { RoomEnemyType::Floater }
            n => { RoomEnemyType::Other(n) }
        }
    }
}

impl RoomEntityDirection {
    pub fn value(self) -> u8 {
        match self {
            RoomEntityDirection::Right => { 0 }
            RoomEntityDirection::Left => { 1 }
        }
    }
}

impl From<u8> for RoomEntityDirection {
    fn from(value: u8) -> Self {
        if value == 0 {
            RoomEntityDirection::Right
        } else {
            RoomEntityDirection::Left
        }
    }
}

#[derive(Clone, std::hash::Hash)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: DataAssetId,
}

#[derive(Clone, std::hash::Hash)]
pub enum RoomTriggerType {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    PlayerSpawn { direction: RoomEntityDirection },
    EnemySpawn { animation_id: DataAssetId, enemy_type: RoomEnemyType, direction: RoomEntityDirection },
    Door { dest_room_id: DataAssetId, dest_trigger_id: u16 },
    Trap { width: u16, height: u16, trap_type: u16 },
    WallButton { width: u16, height: u16, effect_id: u16, required_collectable_id: u16 },
    FloorButton { width: u16, height: u16, effect_id: u16 },
    UnblockEffect { width: u16, height: u16, effect_id: u16 },
    DisableAnimationEffect { width: u16, height: u16, effect_id: u16 },
    GetUpgrade { upgrade_id: u16 },
    GetCollectable { collectable_id: u16 },
    GetPickup { pickup_id: u16 },
}

#[derive(Clone, std::hash::Hash)]
pub struct RoomTrigger {
    pub trigger_id: u16,
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub trigger_type: RoomTriggerType,
}

#[derive(std::hash::Hash)]
pub struct Room {
    pub asset: super::DataAsset,
    pub maps: Vec<RoomMap>,
    pub triggers: Vec<RoomTrigger>,
    pub has_script: bool,
}

impl Room {
    pub fn new(id: DataAssetId, name: String) -> Self {
        Room {
            asset: super::DataAsset::new(super::DataAssetType::Room, id, name),
            maps: Vec::new(),
            triggers: Vec::new(),
            has_script: false,
        }
    }
}

impl super::DataHashAsset for Room {
    fn data_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;

        self.asset.asset_type.hash(state);
        self.asset.name.hash(state);
        for map in &self.maps {
            map.x.hash(state);
            map.y.hash(state);
        }
        for trigger in &self.triggers {
            trigger.trigger_id.hash(state);
            trigger.name_id.hash(state);
            trigger.x.hash(state);
            trigger.y.hash(state);
            match trigger.trigger_type {
                RoomTriggerType::Unknown { data0, data1, data2, data3 } => {
                    0.hash(state);
                    data0.hash(state);
                    data1.hash(state);
                    data2.hash(state);
                    data3.hash(state);
                }
                RoomTriggerType::PlayerSpawn { direction } => {
                    1.hash(state);
                    direction.hash(state);
                }
                RoomTriggerType::EnemySpawn { enemy_type, direction, .. } => {
                    2.hash(state);
                    enemy_type.hash(state);
                    direction.hash(state);
                }
                RoomTriggerType::Door { dest_trigger_id, .. } => {
                    3.hash(state);
                    dest_trigger_id.hash(state);
                }
                RoomTriggerType::Trap { width, height, trap_type } => {
                    4.hash(state);
                    width.hash(state);
                    height.hash(state);
                    trap_type.hash(state);
                }
                RoomTriggerType::WallButton { width, height, effect_id, required_collectable_id } => {
                    5.hash(state);
                    width.hash(state);
                    height.hash(state);
                    effect_id.hash(state);
                    required_collectable_id.hash(state);
                }
                RoomTriggerType::FloorButton { width, height, effect_id } => {
                    6.hash(state);
                    width.hash(state);
                    height.hash(state);
                    effect_id.hash(state);
                }
                RoomTriggerType::UnblockEffect { width, height, effect_id } => {
                    7.hash(state);
                    width.hash(state);
                    height.hash(state);
                    effect_id.hash(state);
                }
                RoomTriggerType::DisableAnimationEffect { width, height, effect_id } => {
                    8.hash(state);
                    width.hash(state);
                    height.hash(state);
                    effect_id.hash(state);
                }
                RoomTriggerType::GetUpgrade { upgrade_id } => {
                    9.hash(state);
                    upgrade_id.hash(state);
                }
                RoomTriggerType::GetCollectable { collectable_id } => {
                    10.hash(state);
                    collectable_id.hash(state);
                }
                RoomTriggerType::GetPickup { pickup_id } => {
                    11.hash(state);
                    pickup_id.hash(state);
                }
            }
        }
        self.has_script.hash(state);
    }
}

impl super::DuplicableAsset<Room> for Room {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        Room {
            asset: self.asset.duplicate(dup_id, dup_name),
            maps: self.maps.clone(),
            triggers: self.triggers.clone(),
            has_script: self.has_script,
        }
    }
}

impl super::GenericAsset for Room {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: num_maps(2) + num_triggers(2) + maps<ptr>(4) + triggers<ptr>(4)
        let header = 2 + 2 + 2 * 4;

        // map[0..num_maps]: x(2) + y(2) + map<ptr>(4)
        let maps = self.maps.len() * (2 + 2 + 4);

        // trigger[0..num_triggers]: type(4) + x(2) + y(2) + data[0..4](4)
        let triggers = self.triggers.len() * (4 + 2 + 2 + 4 * 4);

        header + maps + triggers
    }
}

pub enum RoomTriggerTypeIdent {
    Unknown,
    PlayerSpawn,
    EnemySpawn,
    Door,
    Trap,
    WallButton,
    FloorButton,
    UnblockEffect,
    DisableAnimationEffect,
    GetUpgrade,
    GetCollectable,
    GetPickup,
}

impl RoomTriggerTypeIdent {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown {..} => { RoomTriggerTypeIdent::Unknown }
            RoomTriggerType::Door{..} => { RoomTriggerTypeIdent::Door }
            RoomTriggerType::PlayerSpawn{..} => { RoomTriggerTypeIdent::PlayerSpawn }
            RoomTriggerType::EnemySpawn{..} => { RoomTriggerTypeIdent::EnemySpawn }
            RoomTriggerType::Trap {..} => { RoomTriggerTypeIdent::Trap }
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeIdent::WallButton }
            RoomTriggerType::FloorButton {..} => { RoomTriggerTypeIdent::FloorButton }
            RoomTriggerType::UnblockEffect {..} => { RoomTriggerTypeIdent::UnblockEffect }
            RoomTriggerType::DisableAnimationEffect {..} => { RoomTriggerTypeIdent::DisableAnimationEffect }
            RoomTriggerType::GetUpgrade {..} => { RoomTriggerTypeIdent::GetUpgrade }
            RoomTriggerType::GetCollectable {..} => { RoomTriggerTypeIdent::GetCollectable }
            RoomTriggerType::GetPickup {..} => { RoomTriggerTypeIdent::GetPickup }
        }
    }

    pub fn enum_ident(&self) -> &'static str {
        match self {
            RoomTriggerTypeIdent::Unknown => { "ROOM_TRIGGER_TYPE_ANY" }
            RoomTriggerTypeIdent::Door => { "ROOM_TRIGGER_TYPE_DOOR" }
            RoomTriggerTypeIdent::PlayerSpawn => { "ROOM_TRIGGER_TYPE_PLAYER_SPAWN" }
            RoomTriggerTypeIdent::EnemySpawn => { "ROOM_TRIGGER_TYPE_ENEMY_SPAWN" }
            RoomTriggerTypeIdent::Trap => { "ROOM_TRIGGER_TYPE_TRAP" }
            RoomTriggerTypeIdent::WallButton => { "ROOM_TRIGGER_TYPE_WALL_BUTTON" }
            RoomTriggerTypeIdent::FloorButton => { "ROOM_TRIGGER_TYPE_FLOOR_BUTTON" }
            RoomTriggerTypeIdent::UnblockEffect => { "ROOM_TRIGGER_TYPE_UNBLOCK_EFFECT" }
            RoomTriggerTypeIdent::DisableAnimationEffect => { "ROOM_TRIGGER_TYPE_DISABLE_ANIMATION_EFFECT" }
            RoomTriggerTypeIdent::GetUpgrade => { "ROOM_TRIGGER_TYPE_GET_UPGRADE" }
            RoomTriggerTypeIdent::GetCollectable => { "ROOM_TRIGGER_TYPE_GET_COLLECTABLE" }
            RoomTriggerTypeIdent::GetPickup => { "ROOM_TRIGGER_TYPE_GET_PICKUP" }
        }
    }

    pub fn matches_enum_ident(&self, enum_ident: &str, prefix: &str) -> bool {
        let req_enum_ident = self.enum_ident();
        enum_ident.len() == prefix.len() + req_enum_ident.len() &&
            enum_ident[..prefix.len()].eq_ignore_ascii_case(prefix) &&
            enum_ident[prefix.len()..].eq_ignore_ascii_case(req_enum_ident)
    }
}
