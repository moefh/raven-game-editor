use super::DataAssetId;

#[derive(Clone, std::hash::Hash)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: DataAssetId,
}

#[derive(Clone, std::hash::Hash)]
pub enum RoomEntityType {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    Enemy { animation_id: DataAssetId } ,
}

#[derive(Clone, std::hash::Hash)]
pub struct RoomEntity {
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub entity_type: RoomEntityType,
}

#[derive(Clone, std::hash::Hash)]
pub enum RoomTriggerType {
    Unknown { data0: u16, data1: u16, data2: u16, data3: u16 },
    Door { room_id: DataAssetId, door_id: u16 },
}

#[derive(Clone, std::hash::Hash)]
pub struct RoomTrigger {
    pub name_id: String,
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub trigger_type: RoomTriggerType,
}

#[derive(std::hash::Hash)]
pub struct Room {
    pub asset: super::DataAsset,
    pub maps: Vec<RoomMap>,
    pub entities: Vec<RoomEntity>,
    pub triggers: Vec<RoomTrigger>,
}

impl Room {
    pub fn new(id: DataAssetId, name: String) -> Self {
        Room {
            asset: super::DataAsset::new(super::DataAssetType::Room, id, name),
            maps: Vec::new(),
            triggers: Vec::new(),
            entities: Vec::new(),
        }
    }
}

impl super::DuplicableAsset<Room> for Room {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        Room {
            asset: self.asset.duplicate(dup_id, dup_name),
            maps: self.maps.clone(),
            triggers: self.triggers.clone(),
            entities: self.entities.clone(),
        }
    }
}

impl super::GenericAsset for Room {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: num_maps(1) + num_entities(1) + num_triggers(1) + pad(1) +
        //         maps<ptr>(4) + entities<ptr>(4) + triggers<ptr>(4)
        let header = 4usize + 3usize * 4usize;

        // map[0..num_maps]: x(2) + y(2) + map<ptr>(4)
        let maps = self.maps.len() * (2usize * 2usize + 4usize);

        // entity[0..num_entities]: x(2) + y(2) + anim<ptr>(4) + data[0..4](2)
        let entities = self.entities.len() * (2usize * 2usize + 4usize + 4usize * 2usize);

        // trigger[0..num_triggers]: x(2) + y(2) + w(2) + h(2) + data[0..4](2)
        let triggers = self.triggers.len() * (4usize * 2usize + 4usize * 2usize);

        header + maps + entities + triggers
    }
}

#[allow(unused)]
pub trait RoomItem {
    fn name_id(&self) -> &str;
    fn x(&self) -> i16;
    fn y(&self) -> i16;
}

impl RoomItem for RoomEntity {
    fn name_id(&self) -> &str { &self.name_id }
    fn x(&self) -> i16 { self.x }
    fn y(&self) -> i16 { self.y }
}

impl RoomItem for RoomTrigger {
    fn name_id(&self) -> &str { &self.name_id }
    fn x(&self) -> i16 { self.x }
    fn y(&self) -> i16 { self.y }
}
