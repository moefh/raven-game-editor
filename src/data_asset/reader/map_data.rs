use std::io::Result;

use super::{
    ReaderAssetIndex,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    AssetIdCollection,
    MapData,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub tileset_ref: ReaderAssetIndex,
    pub width: u32,
    pub height: u32,
    pub para_width: u32,
    pub para_height: u32,
    pub tiles: Vec<u8>,
}

impl CreationData {
    pub fn into_map(self, asset_ids: &AssetIdCollection) -> Result<MapData> {
        let fg_size = (self.width * self.height) as usize;
        let fg_tiles = Vec::from(&self.tiles[0..fg_size]);
        let bg_tiles = Vec::from(&self.tiles[fg_size..2*fg_size]);
        let fx_tiles = Vec::from(&self.tiles[2*fg_size..3*fg_size]);
        let para_tiles = Vec::from(&self.tiles[3*fg_size..]);

        Ok(MapData {
            asset: DataAsset::new(DataAssetType::MapData, self.asset_id, self.name),
            tileset_id: self.tileset_ref.get_asset_id(&asset_ids.tilesets)?,
            width: self.width,
            height: self.height,
            para_width: self.para_width,
            para_height: self.para_height,
            fg_tiles,
            bg_tiles,
            fx_tiles,
            para_tiles,
        })
    }
}
