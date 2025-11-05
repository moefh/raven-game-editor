pub struct Tileset {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub num_tiles: u32,
    pub data: Vec<u32>,
}

impl Tileset {

    pub fn new(asset: super::DataAsset) -> Self {
        Tileset {
            asset,
            width: 0,
            height: 0,
            stride: 0,
            num_tiles: 0,
            data: Vec::new(),
        }
    }
    
}
