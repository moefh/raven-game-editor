use crate::data_store;
use crate::data_store::DataAssetId;

pub trait AssetIdHolder {
    fn get_asset_id(&self) -> DataAssetId;
}

impl AssetIdHolder for data_store::Sprite {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_store::Tileset {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_store::PalSprite {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}

impl AssetIdHolder for data_store::Font {
    fn get_asset_id(&self) -> DataAssetId { self.asset.id }
}
