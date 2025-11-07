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

impl super::GenericAsset for MapData {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: u16(2) * (w,h,bg_w,bg_h) + ptr(4) * (tileset,tile_data)
        let header = 2usize * 4usize + 4usize * 2usize;

        // tile_data:
        //   foreground: w * h * [u8(1) * (fg,clip,fx)]
        //   background: bg_w * bg_h * [u8(1) * (bg)]
        let fore_tiles = (self.width as usize) * (self.height as usize) * 3usize;
        let back_tiles = (self.bg_width as usize) * (self.bg_height as usize);

        header + fore_tiles + back_tiles
    }
}
