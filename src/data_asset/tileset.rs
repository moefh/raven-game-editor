pub const TILE_SIZE: u32 = 16;

pub struct Tileset {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub data: Vec<u8>,
}

pub struct CreationData<'a> {
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub data: &'a [u32],
}

impl Tileset {
    pub const TILE_SIZE: u32 = TILE_SIZE;

    pub fn new(asset: super::DataAsset) -> Self {
        Tileset {
            asset,
            width: TILE_SIZE,
            height: TILE_SIZE,
            num_tiles: 10,
            data: vec![0x3f; (TILE_SIZE*TILE_SIZE*10) as usize],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Tileset {
            asset,
            width: data.width,
            height: data.height,
            num_tiles: data.num_tiles,
            data: super::image_pixels_u32_to_u8(data.data, data.width, data.height, data.num_tiles),
        }
    }

}

impl super::GenericAsset for Tileset {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: w(4) + h(4) + stride(4) + num_tiles(4) + data<ptr>(4)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_tiles
        let image = (4 * self.width.div_ceil(4) * self.height * self.num_tiles) as usize;

        header + image
    }
}
