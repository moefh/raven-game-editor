use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Tileset,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub pixels: Vec<u8>,
}

impl CreationData {
    pub fn into_tileset(self) -> Tileset {
        Tileset {
            asset: DataAsset::new(DataAssetType::Tileset, self.asset_id, self.name),
            width: self.width,
            height: self.height,
            num_tiles: self.num_tiles,
            data: self.pixels,
        }
    }
}
