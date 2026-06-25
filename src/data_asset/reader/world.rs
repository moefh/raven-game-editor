use std::io::Result;

use super::ReaderAssetReference;
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    AssetIdCollection,
    World,
    WorldRegion,
};

pub struct RegionCreationData {
    pub name: String,
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub rooms: Vec<ReaderAssetReference>,
    pub block_bitmap: Vec<u32>,
    pub blocks: Vec<u8>,
    pub block_pos: super::TokenPosition,
}

impl RegionCreationData {
    fn into_world_region(self, asset_ids: &AssetIdCollection) -> Result<WorldRegion> {
        let rooms: Result<Vec<_>> = self.rooms.into_iter().map(|room_ref| room_ref.get_asset_id(&asset_ids.rooms)).collect();
        let blocks = Self::conv_blocks(self.width, self.height, self.block_bitmap, self.blocks, self.block_pos);
        Ok(WorldRegion {
            name: self.name,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            rooms: rooms?,
            blocks: blocks?,
        })
    }

    fn conv_blocks(width: u8, height: u8, block_bitmap: Vec<u32>, blocks: Vec<u8>, block_pos: super::TokenPosition) -> Result<Vec<Option<u8>>> {
        let mut ret_blocks = vec![None; WorldRegion::MAX_HEIGHT as usize * WorldRegion::BLOCK_STRIDE];
        let mut block_bitmap_index = 0;
        let mut block_index = 0;
        for y in 0..height as usize {
            for x in 0..width as usize {
                if (block_bitmap[block_bitmap_index/32] & (1 << (block_bitmap_index%32))) != 0 {
                    let block = blocks.get(block_index).copied();
                    if block.is_some() {
                        ret_blocks[y * WorldRegion::BLOCK_STRIDE + x] = block;
                    } else {
                        return super::error(format!("region block is too small: {} <= {}",
                                                    blocks.len(), block_index), block_pos);
                    }
                    block_index += 1;
                }
                block_bitmap_index += 1;
            }
        }
        Ok(ret_blocks)
    }
}

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub regions: Vec<RegionCreationData>,
}

impl CreationData {
    pub fn into_world(self, asset_ids: &AssetIdCollection) -> Result<World> {
        let regions: Result<Vec<_>> = self.regions.into_iter().map(|r| r.into_world_region(asset_ids)).collect();
        Ok(World {
            asset: DataAsset::new(DataAssetType::Room, self.asset_id, self.name),
            regions: regions?,
        })
    }
}
