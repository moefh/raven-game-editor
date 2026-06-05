use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Sprite,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub pixels: Vec<u8>,
}

impl CreationData {
    pub fn into_sprite(self) -> Sprite {
        Sprite {
            asset: DataAsset::new(DataAssetType::Sprite, self.asset_id, self.name),
            width: self.width,
            height: self.height,
            num_frames: self.num_frames,
            data: self.pixels,
        }
    }
}
