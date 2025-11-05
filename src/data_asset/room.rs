#[allow(unused)]
#[derive(Clone)]
pub struct RoomMap {
    pub name: String,
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
    pub width: u16,
    pub height: u16,
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

impl Room {

    pub fn new(asset: super::DataAsset) -> Self {
        Room {
            asset,
            maps: Vec::new(),
            triggers: Vec::new(),
            entities: Vec::new(),
        }
    }
    
}
