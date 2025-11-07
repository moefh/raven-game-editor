pub struct Sprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub num_frames: u32,
    pub data: Vec<u32>,
}

pub struct CreationData<'a> {
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub data: &'a [u32],
}

impl Sprite {
    pub fn new(asset: super::DataAsset) -> Self {
        Sprite {
            asset,
            width: 32,
            height: 32,
            stride: 32/4,
            num_frames: 8,
            data: vec![0x3f3f3f3f; 32*32/4*8],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Sprite {
            asset,
            width: data.width,
            height: data.height,
            stride: data.width.div_ceil(4),
            num_frames: data.num_frames,
            data: Vec::from(data.data),
        }
    }
}

impl super::GenericAsset for Sprite {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: 4 * (w,h,stride,num_frames,ptr)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_frames * num_mirrors(2)
        let image = 4usize * (self.stride as usize) * (self.height as usize) * (self.num_frames as usize) * 2usize;

        header + image  // include mirror images
    }
}

impl super::ImageCollectionAsset for Sprite {
    fn asset_id(&self) -> super::DataAssetId { self.asset.id }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn stride(&self) -> u32 { self.stride }
    fn num_items(&self) -> u32 { self.num_frames }
    fn data(&self) -> &[u32] { &self.data }
}
