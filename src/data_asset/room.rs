use super::DataAssetId;

#[derive(Clone, std::hash::Hash)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: DataAssetId,
}

#[derive(Clone, std::hash::Hash)]
pub enum RoomTriggerType {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    PlayerSpawn { direction: u8 },
    EnemySpawn { animation_id: DataAssetId },
    Door { dest_room_id: DataAssetId, dest_trigger_id: u16 },
    Trap { width: u16, height: u16, trap_type: u16 },
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
}

impl Room {
    pub fn new(id: DataAssetId, name: String) -> Self {
        Room {
            asset: super::DataAsset::new(super::DataAssetType::Room, id, name),
            maps: Vec::new(),
            triggers: Vec::new(),
        }
    }
}

impl super::DuplicableAsset<Room> for Room {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        Room {
            asset: self.asset.duplicate(dup_id, dup_name),
            maps: self.maps.clone(),
            triggers: self.triggers.clone(),
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
}

impl RoomTriggerTypeIdent {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown {..} => { RoomTriggerTypeIdent::Unknown }
            RoomTriggerType::Door{..} => { RoomTriggerTypeIdent::Door }
            RoomTriggerType::PlayerSpawn{..} => { RoomTriggerTypeIdent::PlayerSpawn }
            RoomTriggerType::EnemySpawn{..} => { RoomTriggerTypeIdent::EnemySpawn }
            RoomTriggerType::Trap {..} => { RoomTriggerTypeIdent::Trap }
        }
    }

    pub fn enum_ident(&self) -> &'static str {
        match self {
            RoomTriggerTypeIdent::Unknown => { "ROOM_TRIGGER_TYPE_ANY" }
            RoomTriggerTypeIdent::Door => { "ROOM_TRIGGER_TYPE_DOOR" }
            RoomTriggerTypeIdent::PlayerSpawn => { "ROOM_TRIGGER_TYPE_PLAYER_SPAWN" }
            RoomTriggerTypeIdent::EnemySpawn => { "ROOM_TRIGGER_TYPE_ENEMY_SPAWN" }
            RoomTriggerTypeIdent::Trap => { "ROOM_TRIGGER_TYPE_TRAP" }
        }
    }

    pub fn matches_enum_ident(&self, enum_ident: &str, prefix: &str) -> bool {
        let req_enum_ident = self.enum_ident();
        enum_ident.len() == prefix.len() + req_enum_ident.len() &&
            enum_ident[..prefix.len()].eq_ignore_ascii_case(prefix) &&
            enum_ident[prefix.len()..].eq_ignore_ascii_case(req_enum_ident)
    }
}
