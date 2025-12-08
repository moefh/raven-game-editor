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

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 32;
        let height = 32;
        let num_frames = 8;
        Sprite {
            asset: super::DataAsset::new(super::DataAssetType::Sprite, id, name),
            width,
            height,
            num_frames,
            data: vec![0x3f; (width*height*num_frames) as usize],
        }
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        Sprite {
            asset: super::DataAsset::new(super::DataAssetType::Sprite, id, name),
            width: data.width,
            height: data.height,
            num_frames: data.num_frames,
            data: super::image_u32_to_pixels(data.data, data.width, data.height, data.num_frames),
        }
    }
}

impl super::GenericAsset for Sprite {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: w(4) + h(4) + stride(4) + num_frames(4) + data<ptr>(4)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_frames
        let image = (4 * self.width.div_ceil(4) * self.height * self.num_frames) as usize;

        header + image * 2  // include mirror images
    }
}
