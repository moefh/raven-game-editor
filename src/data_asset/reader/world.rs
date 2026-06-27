use std::io::Result;

use super::{
    error,
    Value,
    AssetDef,
    ProjectData,
    TokenPosition,
};
use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    World,
    WorldRegion,
};

fn conv_blocks(width: u8, height: u8, block_bitmap: &[u32], blocks: &[u8], block_pos: super::TokenPosition) -> Result<Vec<Option<u8>>> {
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
                    return error(format!("region block is too small: {} <= {}",
                                         blocks.len(), block_index), block_pos);
                }
                block_index += 1;
            }
            block_bitmap_index += 1;
        }
    }
    Ok(ret_blocks)
}

fn conv_region(region: &Value, name: String, project_data: &ProjectData, pos: TokenPosition) -> Result<WorldRegion> {
    if let Value::Struct(value) = region && let [
        Value::U8(x),
        Value::U8(y),
        Value::U8(width),
        Value::U8(height),
        Value::ArrayRef(block_bitmap_array),
        Value::ArrayRef(blocks_array),
        Value::ArrayRef(room_indices_array),
    ] = &value[..] {
        let block_bitmap = block_bitmap_array.get_u32_array(project_data)?;
        let blocks_data = blocks_array.get_u8_array(project_data)?;
        let blocks = conv_blocks(*width, *height, block_bitmap, blocks_data, blocks_array.pos)?;

        let mut rooms = Vec::new();
        for room_index in room_indices_array.get_u16_array(project_data)? {
            match project_data.asset_ids.get("rooms").and_then(|room_ids| room_ids.get(*room_index as usize)) {
                Some(id) => { rooms.push(*id) }
                None => {
                    return error(format!("invalid world region room index: {}", room_index), room_indices_array.pos);
                }
            }
        }

        Ok(WorldRegion {
            name,
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            rooms,
            blocks,
        })
    } else {
        error(format!("bad region data: {:?}", region), pos)
    }
}

pub fn create(asset_id: DataAssetId, asset_def: &AssetDef, project_data: &ProjectData) -> Result<World> {
    if let Value::Struct(value) = &asset_def.value && let [
        Value::U16(num_regions),
        Value::ArrayRef(regions_array),
    ] = &value[..] {
        let name = project_data.extract_asset_name("world_regions_", regions_array)?;
        let regions_data = regions_array.get_struct_array(project_data)?;
        let regions = regions_data.iter().enumerate().map(|(index, region)| {
            let name = project_data
                .get_asset_data_name(index, "WORLD", name, "REGION")
                .unwrap_or_else(|| format!("region_{}", index));
            conv_region(region, name, project_data, asset_def.pos)
        }).collect::<Result<Vec<_>>>()?;

        if *num_regions as usize != regions.len() {
            return error(format!("invalid number of world regions: expected {}, got {}",
                                 *num_regions, regions.len()), regions_array.pos);
        }

        Ok(World {
            asset: DataAsset::new(DataAssetType::World, asset_id, DataAsset::identifier_to_name(name)),
            regions,
        })
    } else {
        error(format!("bad world data: {:?}", asset_def.value), asset_def.pos)
    }
}
