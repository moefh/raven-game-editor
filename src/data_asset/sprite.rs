#[derive(std::hash::Hash)]
pub struct Sprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub data: Vec<u8>,
}

impl Sprite {
    pub const MIRROR_FRAMES: bool = true;
    pub const EMPTY_COLOR: u8 = 0xff;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 32;
        let height = 32;
        let num_frames = 8;
        Sprite {
            asset: super::DataAsset::new(super::DataAssetType::Sprite, id, name),
            width,
            height,
            num_frames,
            data: vec![Self::EMPTY_COLOR; (width*height*num_frames) as usize],
        }
    }
}

impl super::DuplicableAsset<Sprite> for Sprite {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        Sprite {
            asset: self.asset.duplicate(dup_id, dup_name),
            width: self.width,
            height: self.height,
            num_frames: self.num_frames,
            data: self.data.clone(),
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
