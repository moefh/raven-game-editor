use std::io::{Result, Error};
use std::collections::HashMap;

use super::{
    ProjectDataWriter,
    IdentStore,
};
use super::super::{
    DataAssetId,
    DataAssetType,
    World,
    WorldRegion,
};

fn write_world_region_block_bitmap(
    writer: &ProjectDataWriter,
    region: &WorldRegion,
    region_index: usize,
    world_name_id: &str
) -> Result<()> {
    writer.write(format!(
        "static const uint32_t {}_world_{}_region_{}_block_bitmap[] = {{",
        writer.ident.prefix_lower,
        world_name_id,
        region_index
    ));
    let mut bits = 0;
    let mut num_bits = 0;
    let mut num_chunks = 0;
    for y in 0..region.height as usize {
        for x in 0..region.width as usize {
            if region.blocks[y * WorldRegion::BLOCK_STRIDE + x].is_some() {
                bits |= 1 << num_bits;
            }
            num_bits += 1;
            if num_bits == 32 {
                if num_chunks % 8 == 0 { writer.write("\n    "); }
                writer.write(format!("{:#010x},", bits));
                bits = 0;
                num_bits = 0;
                num_chunks += 1;
            }
        }
    }
    if num_bits != 0 {
        if num_chunks % 8 == 0 { writer.write("\n    "); }
        writer.write(format!("{:#010x},", bits));
    }

    writer.write("\n};\n\n");
    Ok(())
}

fn write_world_region_blocks(
    writer: &ProjectDataWriter,
    region: &WorldRegion,
    region_index: usize,
    world_name_id: &str
) -> Result<()> {
    writer.write(format!(
        "static const uint8_t {}_world_{}_region_{}_blocks[] = {{",
        writer.ident.prefix_lower,
        world_name_id,
        region_index
    ));

    let mut num_blocks_written = 0;
    for y in 0..region.height as usize {
        for x in 0..region.width as usize {
            if let Some(block) = region.blocks[y * WorldRegion::BLOCK_STRIDE + x] {
                if num_blocks_written % 16 == 0 { writer.write("\n    "); }
                writer.write(format!("{:#04x},", block));
                num_blocks_written += 1;
            }
        }
    }
    writer.write("\n};\n\n");
    Ok(())
}

fn write_world_region_room_indices(
    writer: &ProjectDataWriter,
    region: &WorldRegion,
    region_index: usize,
    world_name_id: &str
) -> Result<()> {
    writer.write(format!(
        "static const uint16_t {}_world_{}_region_{}_room_indices[] = {{",
        writer.ident.prefix_lower,
        world_name_id,
        region_index
    ));
    for (index, &room_id) in region.rooms.iter().enumerate() {
        if index % 16 == 0 { writer.write("\n    "); }
        let room_index = writer.ident.get_asset_index(DataAssetType::Room, room_id)?;
        writer.write(format!("{},", room_index));
    }
    writer.write("\n};\n\n");
    Ok(())
}

fn write_world_regions(writer: &ProjectDataWriter, world: &World, name_id: &str) -> Result<()> {
    for (region_index, region) in world.regions.iter().enumerate() {
        write_world_region_block_bitmap(writer, region, region_index, name_id)?;
        write_world_region_blocks(writer, region, region_index, name_id)?;
        write_world_region_room_indices(writer, region, region_index, name_id)?;
    }

    writer.write(format!(
        "static const struct {}_WORLD_REGION {}_world_regions_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for (region_index, region) in world.regions.iter().enumerate() {
        writer.write("  {\n");
        writer.write(format!("    {}, {}, {}, {},\n", region.x, region.y, region.width, region.height));
        writer.write(format!("    {}_world_{}_region_{}_block_bitmap,\n", writer.ident.prefix_lower, name_id, region_index));
        writer.write(format!("    {}_world_{}_region_{}_blocks,\n", writer.ident.prefix_lower, name_id, region_index));
        writer.write(format!("    {}_world_{}_region_{}_room_indices\n", writer.ident.prefix_lower, name_id, region_index));
        writer.write("  },\n");
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_worlds(writer: &ProjectDataWriter, world_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === WORLDS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in world_ids.iter() {
        if let Some(world) = writer.store.assets.worlds.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::World, *id)?;
            write_world_regions(writer, world, name_id)?;
        }
    }

    writer.log(format!("-> writing {} worlds", writer.store.asset_ids.rooms.len()));
    writer.write(format!(
        "const struct {}_WORLD {}_worlds[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in world_ids.iter() {
        if let Some(world) = writer.store.assets.worlds.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::World, *id)?;
            writer.write(format!("  {{ {}, {}_world_regions_{} }},\n",
                world.regions.len(),
                writer.ident.prefix_lower, name_id));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_world_region_names(writer: &ProjectDataWriter, world: &World, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_WORLD_{}_REGION_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, reg) in world.regions.iter().enumerate() {
        let use_name_id = if reg.name == "names" { "names0" } else { &reg.name };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let reg_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of region {} world {}", index, world.asset.id))
        })?;
        writer.write(format!("  {}_WORLD_{}_REGION_{},\n", writer.ident.prefix_upper, name_id_upper, reg_name_id.to_ascii_uppercase()));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_world_item_names(writer: &ProjectDataWriter, world_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === WORLD REGION NAMES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in world_ids.iter() {
        if let Some(world) = writer.store.assets.worlds.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::World, *id)?;
            let name_id_upper = name_id.to_ascii_uppercase();
            write_world_region_names(writer, world, &name_id_upper)?;
        }
    }

    Ok(())
}
