pub const TILE_SIZE: u32 = 16;

pub struct Tileset {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub num_tiles: u32,
    pub data: Vec<u32>,
}

pub struct CreationData<'a> {
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub data: &'a [u32],
}

impl Tileset {
    pub fn new(asset: super::DataAsset) -> Self {
        Tileset {
            asset,
            width: TILE_SIZE,
            height: TILE_SIZE,
            stride: TILE_SIZE/4,
            num_tiles: 10,
            data: vec![0x3f3f3f3f; (TILE_SIZE*TILE_SIZE/4*10) as usize],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Tileset {
            asset,
            width: data.width,
            height: data.height,
            stride: data.width.div_ceil(4),
            num_tiles: data.num_tiles,
            data: Vec::from(data.data),
        }
    }

}

impl super::GenericAsset for Tileset {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: 4 * (w,h,stride,num_frames,ptr)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_tiles
        let image = 4usize * (self.stride as usize) * (self.height as usize) * (self.num_tiles as usize);

        header + image
    }
}

impl super::ImageCollectionAsset for Tileset {
    fn asset_id(&self) -> super::DataAssetId { self.asset.id }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn stride(&self) -> u32 { self.stride }
    fn num_items(&self) -> u32 { self.num_tiles }
    fn data(&self) -> &[u32] { &self.data }
}
