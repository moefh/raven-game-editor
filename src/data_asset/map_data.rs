#[allow(dead_code)]
pub struct MapData {
    pub asset: super::DataAsset,
    pub tileset_id: super::DataAssetId,
    pub width: u32,
    pub height: u32,
    pub bg_width: u32,
    pub bg_height: u32,
    pub fg_tiles: Vec<u8>,
    pub clip_tiles: Vec<u8>,
    pub fx_tiles: Vec<u8>,
    pub bg_tiles: Vec<u8>,
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
        let width = 24;
        let height = 24;
        let bg_width = 24;
        let bg_height = 24;
        MapData {
            asset,
            tileset_id,
            width,
            height,
            bg_width,
            bg_height,
            fg_tiles: vec![0xff; (width * height) as usize],
            clip_tiles: vec![0xff; (width * height) as usize],
            fx_tiles: vec![0xff; (width * height) as usize],
            bg_tiles: vec![0; (bg_width * bg_height) as usize],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        let fg_size = (data.width * data.height) as usize;
        let fg_tiles = Vec::from(&data.tiles[0..fg_size]);
        let clip_tiles = Vec::from(&data.tiles[fg_size..2*fg_size]);
        let fx_tiles = Vec::from(&data.tiles[2*fg_size..3*fg_size]);
        let bg_tiles = Vec::from(&data.tiles[3*fg_size..]);
        MapData {
            asset,
            tileset_id: data.tileset_id,
            width: data.width,
            height: data.height,
            bg_width: data.bg_width,
            bg_height: data.bg_height,
            fg_tiles,
            clip_tiles,
            fx_tiles,
            bg_tiles,
        }
    }
}

impl super::GenericAsset for MapData {
    fn asset(&self) -> &super::DataAsset { &self.asset }

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
