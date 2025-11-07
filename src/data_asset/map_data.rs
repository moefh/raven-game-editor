#[allow(dead_code)]
pub struct MapData {
    pub asset: super::DataAsset,
    pub tileset_id: super::DataAssetId,
    pub width: u32,
    pub height: u32,
    pub bg_width: u32,
    pub bg_height: u32,
    pub tiles: Vec<u8>,
}

pub struct CreationData<'a> {
    pub tileset_id: super::DataAssetId,
    pub width: u32,
    pub height: u32,
    pub bg_width: u32,
    pub bg_height: u32,
    pub tiles: &'a [u8],
}

impl MapData {
    pub fn new(asset: super::DataAsset, tileset_id: super::DataAssetId) -> Self {
        MapData {
            asset,
            tileset_id,
            width: 24,
            height: 24,
            bg_width: 24,
            bg_height: 24,
            tiles: vec![0; 24*24*4],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        MapData {
            asset,
            tileset_id: data.tileset_id,
            width: data.width,
            height: data.height,
            bg_width: data.bg_width,
            bg_height: data.bg_height,
            tiles: Vec::from(data.tiles),
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
