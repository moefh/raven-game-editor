use super::DataAssetId;

#[derive(Copy, Clone, PartialEq)]
pub enum RoomEntityDirection {
    Right,
    Left,
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

#[derive(Copy, Clone, PartialEq)]
pub enum RoomTriggerMapLayer {
    Background2,
    Background,
    Foreground,
}

impl RoomTriggerMapLayer {
    pub fn value(self) -> u8 {
        match self {
            RoomTriggerMapLayer::Background2 => { 0 }
            RoomTriggerMapLayer::Background => { 1 }
            RoomTriggerMapLayer::Foreground => { 2 }
        }
    }
}

impl From<u8> for RoomTriggerMapLayer {
    fn from(value: u8) -> Self {
        match value {
            0 => { RoomTriggerMapLayer::Background2 }
            1 => { RoomTriggerMapLayer::Background }
            _ => { RoomTriggerMapLayer::Foreground }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Clone)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: DataAssetId,
}

#[derive(Clone)]
pub enum RoomTriggerType {
    Unknown {
        data0: u16,
        data1: u16,
        data2: u16,
        data3: u16,
        data4: u16,
        data5: u16
    },
    PlayerSpawn {
        direction: RoomEntityDirection
    },
    EnemySpawn {
        animation_id: DataAssetId,
        enemy_id: u16,
        enemy_type: RoomEnemyType,
        direction: RoomEntityDirection,
        req_flag_id: u16
    },
    Door {
        dest_room_id: DataAssetId,
        dest_trigger_id: u16,
        req_flag_id: u16
    },
    WallButton {
        width: u16,
        height: u16,
        set_effect_id: u16,
        set_flag_id: u16,
        req_flag_id: u16,
        req_collectable_id: u16
    },
    DetectorButton {
        width: u16,
        height: u16,
        set_effect_id: u16,
        set_flag_id: u16,
        req_flag_id: u16,
        req_collectable_id: u16
    },
    UnblockTiles {
        width: u16,
        height: u16,
        on_effect_id: u16
    },
    DisableAnimation {
        width: u16,
        height: u16,
        on_effect_id: u16
    },
    ChangeTiles {
        map_index: u16,
        tile_change_index: u16,
        map_layer: RoomTriggerMapLayer,
        on_effect_id: u16
    },
    RemoveEnemy {
        enemy_id: u16,
        on_effect_id: u16
    },
    GetUpgrade {
        get_upgrade_id: u16
    },
    GetCollectable {
        get_collectable_id: u16
    },
    GetPickup {
        get_pickup_id: u16,
        respawn_delay: u16
    },
}

#[derive(Clone)]
pub struct RoomTrigger {
    pub trigger_id: u16,
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub trigger_type: RoomTriggerType,
}

#[derive(Clone)]
pub struct RoomParallax {
    pub tileset_id: Option<DataAssetId>,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u8>,
}

pub struct Room {
    pub asset: super::DataAsset,
    pub maps: Vec<RoomMap>,
    pub triggers: Vec<RoomTrigger>,
    pub has_script: bool,
    pub parallax: RoomParallax,
}

impl Room {
    pub fn new(id: DataAssetId, name: String) -> Self {
        Room {
            asset: super::DataAsset::new(super::DataAssetType::Room, id, name),
            maps: Vec::new(),
            triggers: Vec::new(),
            has_script: false,
            parallax: RoomParallax {
                tileset_id: None,
                width: 0,
                height: 0,
                tiles: Vec::new(),
            },
        }
    }
}

impl super::HashedDataAsset for Room {
    fn data_hash(&self, hasher: &mut super::DataAssetHasher) {
        hasher.add_data_asset(&self.asset);
        if let Some(tileset_id) = self.parallax.tileset_id {
            hasher.add_u8(1);
            hasher.add_asset_id(tileset_id);
            hasher.add_u32(self.parallax.width);
            hasher.add_u32(self.parallax.height);
            hasher.add_bytes(&self.parallax.tiles);
        } else {
            hasher.add_u8(0);
        }
        for map in &self.maps {
            hasher.add_asset_id(map.map_id);
            hasher.add_u16(map.x);
            hasher.add_u16(map.y);
        }
        for trigger in &self.triggers {
            hasher.add_u16(trigger.trigger_id);
            hasher.add_str(&trigger.name_id);
            hasher.add_i16(trigger.x);
            hasher.add_i16(trigger.y);
            match trigger.trigger_type {
                RoomTriggerType::Unknown { data0, data1, data2, data3, data4, data5 } => {
                    hasher.add_u8(0);
                    hasher.add_u16(data0);
                    hasher.add_u16(data1);
                    hasher.add_u16(data2);
                    hasher.add_u16(data3);
                    hasher.add_u16(data4);
                    hasher.add_u16(data5);
                }
                RoomTriggerType::PlayerSpawn { direction } => {
                    hasher.add_u8(1);
                    hasher.add_u8(direction.value());
                }
                RoomTriggerType::EnemySpawn { animation_id, enemy_id, enemy_type, direction, req_flag_id } => {
                    hasher.add_u8(2);
                    hasher.add_asset_id(animation_id);
                    hasher.add_u16(enemy_id);
                    hasher.add_u16(enemy_type.value());
                    hasher.add_u8(direction.value());
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::Door { dest_room_id, dest_trigger_id, req_flag_id } => {
                    hasher.add_u8(3);
                    hasher.add_asset_id(dest_room_id);
                    hasher.add_u16(dest_trigger_id);
                    hasher.add_u16(req_flag_id);
                }
                RoomTriggerType::WallButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                    hasher.add_u8(5);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(set_flag_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(req_collectable_id);
                }
                RoomTriggerType::DetectorButton { width, height, set_effect_id, set_flag_id, req_flag_id, req_collectable_id } => {
                    hasher.add_u8(6);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(set_effect_id);
                    hasher.add_u16(set_flag_id);
                    hasher.add_u16(req_flag_id);
                    hasher.add_u16(req_collectable_id);
                }
                RoomTriggerType::UnblockTiles { width, height, on_effect_id } => {
                    hasher.add_u8(7);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::DisableAnimation { width, height, on_effect_id } => {
                    hasher.add_u8(8);
                    hasher.add_u16(width);
                    hasher.add_u16(height);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::ChangeTiles { map_index, tile_change_index, map_layer, on_effect_id } => {
                    hasher.add_u8(9);
                    hasher.add_u16(map_index);
                    hasher.add_u16(tile_change_index);
                    hasher.add_u8(map_layer.value());
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::RemoveEnemy { enemy_id, on_effect_id } => {
                    hasher.add_u8(10);
                    hasher.add_u16(enemy_id);
                    hasher.add_u16(on_effect_id);
                }
                RoomTriggerType::GetUpgrade { get_upgrade_id } => {
                    hasher.add_u8(11);
                    hasher.add_u16(get_upgrade_id);
                }
                RoomTriggerType::GetCollectable { get_collectable_id } => {
                    hasher.add_u8(12);
                    hasher.add_u16(get_collectable_id);
                }
                RoomTriggerType::GetPickup { get_pickup_id, respawn_delay } => {
                    hasher.add_u8(13);
                    hasher.add_u16(get_pickup_id);
                    hasher.add_u16(respawn_delay);
                }
            }
        }
        hasher.add_bool(self.has_script);
    }
}

impl super::DuplicableAsset<Room> for Room {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        Room {
            asset: self.asset.duplicate(dup_id, dup_name),
            maps: self.maps.clone(),
            triggers: self.triggers.clone(),
            has_script: self.has_script,
            parallax: self.parallax.clone(),
        }
    }
}

impl super::GenericAsset for Room {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: u16(2) * (num_maps, num_triggers) + ptr(4) * (maps, triggers)
        let header = 2usize * 2 + 4usize * 2;

        // map[0..num_maps]: u16(2) * (x, y) + map<ptr>(4)
        let maps = self.maps.len() * (2usize * 2 + 4usize);

        // trigger[0..num_triggers]: u16(2)*(type,id,x,y) + u16(2)*6
        let triggers = self.triggers.len() * (2usize * 4 + 2usize * 6);

        // parallax:
        //   header: tileset<ptr>(4) + u16(2) * (width, height) + tiles<ptr>(4)
        //   tiles:  w * h * [u8(1)]
        let parallax = 4usize + 2usize * 2 + 4usize + (self.parallax.width as usize * self.parallax.height as usize);

        header + maps + triggers + parallax
    }
}

pub enum RoomTriggerTypeIdent {
    Unknown,
    PlayerSpawn,
    EnemySpawn,
    Door,
    WallButton,
    DetectorButton,
    UnblockTiles,
    DisableAnimation,
    ChangeTiles,
    RemoveEnemy,
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
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeIdent::WallButton }
            RoomTriggerType::DetectorButton {..} => { RoomTriggerTypeIdent::DetectorButton }
            RoomTriggerType::UnblockTiles {..} => { RoomTriggerTypeIdent::UnblockTiles }
            RoomTriggerType::DisableAnimation {..} => { RoomTriggerTypeIdent::DisableAnimation }
            RoomTriggerType::ChangeTiles {..} => { RoomTriggerTypeIdent::ChangeTiles }
            RoomTriggerType::RemoveEnemy {..} => { RoomTriggerTypeIdent::RemoveEnemy }
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
            RoomTriggerTypeIdent::WallButton => { "ROOM_TRIGGER_TYPE_WALL_BUTTON" }
            RoomTriggerTypeIdent::DetectorButton => { "ROOM_TRIGGER_TYPE_DETECTOR_BUTTON" }
            RoomTriggerTypeIdent::UnblockTiles => { "ROOM_TRIGGER_TYPE_UNBLOCK_TILES" }
            RoomTriggerTypeIdent::DisableAnimation => { "ROOM_TRIGGER_TYPE_DISABLE_ANIMATION" }
            RoomTriggerTypeIdent::ChangeTiles => { "ROOM_TRIGGER_TYPE_CHANGE_TILES" }
            RoomTriggerTypeIdent::RemoveEnemy => { "ROOM_TRIGGER_TYPE_REMOVE_ENEMY" }
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
