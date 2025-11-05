pub struct MapData {
    pub asset: super::DataAsset,
    pub tileset_id: super::DataAssetId,
    pub width: u32,
    pub height: u32,
    pub bg_width: u32,
    pub bg_height: u32,
    pub tiles: Vec<u8>,
}

impl MapData {

    pub fn new(asset: super::DataAsset, tileset_id: super::DataAssetId) -> Self {
        MapData {
            asset,
            tileset_id,
            width: 0,
            height: 0,
            bg_width: 0,
            bg_height: 0,
            tiles: Vec::new(),
        }
    }
    
}
