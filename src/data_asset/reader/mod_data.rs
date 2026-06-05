use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    ModData,
    ModSample,
    ModCell,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub samples: Vec<ModSample>,
    pub pattern: Vec<ModCell>,
    pub song_positions: Vec<u8>,
    pub num_channels: u8,
}

impl CreationData {
    pub fn into_mod(self) -> ModData {
        ModData {
            asset: DataAsset::new(DataAssetType::ModData, self.asset_id, self.name),
            num_channels: self.num_channels,
            samples: self.samples,
            pattern: self.pattern,
            song_positions: self.song_positions,
        }
    }
}
