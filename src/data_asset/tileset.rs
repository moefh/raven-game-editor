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
