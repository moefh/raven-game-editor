use crate::data_asset;
use crate::data_asset::DataAssetId;

pub trait AssetIdHolder {
    fn get_asset_id(&self) -> DataAssetId;
}

impl AssetIdHolder for data_asset::Sprite {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_asset::Tileset {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_asset::PalSprite {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_asset::Font {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}
