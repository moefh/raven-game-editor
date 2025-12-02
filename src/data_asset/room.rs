#[allow(unused)]
#[derive(Clone)]
pub struct RoomMap {
    pub x: u16,
    pub y: u16,
    pub map_id: super::DataAssetId,
}

#[allow(unused)]
#[derive(Clone)]
pub struct RoomEntity {
    pub name: String,
    pub x: i16,
    pub y: i16,
    pub animation_id: super::DataAssetId,
    pub data0: u16,
    pub data1: u16,
    pub data2: u16,
    pub data3: u16,
}

#[allow(unused)]
#[derive(Clone)]
pub struct RoomTrigger {
    pub name: String,
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub data0: u16,
    pub data1: u16,
    pub data2: u16,
    pub data3: u16,
}

pub struct Room {
    pub asset: super::DataAsset,
    pub maps: Vec<RoomMap>,
    pub entities: Vec<RoomEntity>,
    pub triggers: Vec<RoomTrigger>,
}

pub struct CreationData<'a> {
    pub maps: &'a [RoomMap],
    pub entities: &'a [RoomEntity],
    pub triggers: &'a [RoomTrigger],
}

impl Room {
    pub fn new(asset: super::DataAsset) -> Self {
        Room {
            asset,
            maps: Vec::new(),
            triggers: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Room {
            asset,
            maps: Vec::from(data.maps),
            triggers: Vec::from(data.triggers),
            entities: Vec::from(data.entities),
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
    fn name(&self) -> &str;
    fn x(&self) -> i16;
    fn y(&self) -> i16;
    fn data0(&self) -> u16;
    fn data1(&self) -> u16;
    fn data2(&self) -> u16;
    fn data3(&self) -> u16;
}

impl RoomItem for RoomEntity {
    fn name(&self) -> &str { &self.name }
    fn x(&self) -> i16 { self.x }
    fn y(&self) -> i16 { self.y }
    fn data0(&self) -> u16 { self.data0 }
    fn data1(&self) -> u16 { self.data1 }
    fn data2(&self) -> u16 { self.data2 }
    fn data3(&self) -> u16 { self.data3 }
}

impl RoomItem for RoomTrigger {
    fn name(&self) -> &str { &self.name }
    fn x(&self) -> i16 { self.x }
    fn y(&self) -> i16 { self.y }
    fn data0(&self) -> u16 { self.data0 }
    fn data1(&self) -> u16 { self.data1 }
    fn data2(&self) -> u16 { self.data2 }
    fn data3(&self) -> u16 { self.data3 }
}
