pub struct Sprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub data: Vec<u8>,
}

pub struct CreationData<'a> {
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub data: &'a [u32],
}

impl Sprite {
    pub const MIRROR_FRAMES: bool = true;

    pub fn new(asset: super::DataAsset) -> Self {
        Sprite {
            asset,
            width: 32,
            height: 32,
            num_frames: 8,
            data: vec![0x3f; 32*32*8],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Sprite {
            asset,
            width: data.width,
            height: data.height,
            num_frames: data.num_frames,
            data: super::image_pixels_u32_to_u8(data.data, data.width, data.height, data.num_frames),
        }
    }
}

impl super::GenericAsset for Sprite {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: w(4) + h(4) + stride(4) + num_frames(4) + data<ptr>(4)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_frames
        let image = (4 * self.width.div_ceil(4) * self.height * self.num_frames) as usize;

        header + image * 2  // include mirror images
    }
}

impl super::ImageCollectionAsset for Sprite {
    fn asset_id(&self) -> super::DataAssetId { self.asset.id }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { self.num_frames }
    fn data(&self) -> &[u8] { &self.data }
    fn data_mut(&mut self) -> &mut [u8] { &mut self.data }
}
