pub struct Sprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub add_mirrors: bool,
    pub data: Vec<u8>,
}

impl Sprite {
    pub const EMPTY_COLOR: u8 = 0;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 32;
        let height = 32;
        let num_frames = 1;
        Sprite {
            asset: super::DataAsset::new(super::DataAssetType::Sprite, id, name),
            width,
            height,
            num_frames,
            add_mirrors: true,
            data: vec![Self::EMPTY_COLOR; (width*height*num_frames) as usize],
        }
    }
}

impl super::HashedDataAsset for Sprite {
    fn data_hash(&self, hasher: &mut super::DataAssetHasher) {
        hasher.add_data_asset(&self.asset);
        hasher.add_u32(self.width);
        hasher.add_u32(self.height);
        hasher.add_u32(self.num_frames);
        hasher.add_bool(self.add_mirrors);
        hasher.add_bytes(&self.data);
    }
}

impl super::DuplicableAsset<Sprite> for Sprite {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        Sprite {
            asset: self.asset.duplicate(dup_id, dup_name),
            width: self.width,
            height: self.height,
            num_frames: self.num_frames,
            add_mirrors: self.add_mirrors,
            data: self.data.clone(),
        }
    }
}

impl super::GenericAsset for Sprite {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: w(4) + h(4) + stride(4) + num_frames(4) + add_mirrors(4) + data<ptr>(4)
        let header = 4usize * 6usize;

        // image: (4*stride) * height * num_frames
        let image = (4 * self.width.div_ceil(4) * self.height * self.num_frames) as usize;

        header + image * if self.add_mirrors { 2 } else { 1 }
    }
}
