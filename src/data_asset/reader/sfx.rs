use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Sfx,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub bits_per_sample: u16,
    pub samples: Vec<i16>,
}

impl CreationData {
    pub fn into_sfx(self) -> Sfx {
        Sfx {
            asset: DataAsset::new(DataAssetType::Sfx, self.asset_id, self.name),
            len: self.len,
            loop_start: self.loop_start,
            loop_len: self.loop_len,
            bits_per_sample: self.bits_per_sample,
            samples: self.samples,
        }
    }
}
