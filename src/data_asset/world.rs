use super::DataAssetId;

#[derive(Clone)]
pub struct WorldRegion {
    pub name: String,
    pub rooms: Vec<DataAssetId>,
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub blocks: Vec<Option<u8>>,
}

impl WorldRegion {
    pub const MAX_WIDTH: u8 = 64;
    pub const MAX_HEIGHT: u8 = 64;
    pub const BLOCK_STRIDE: usize = Self::MAX_WIDTH as usize;

    pub fn new(name: impl Into<String>, x: u8, y: u8, width: u8, height: u8) -> Self {
        WorldRegion {
            name: name.into(),
            x,
            y,
            width,
            height,
            blocks: vec![None; Self::BLOCK_STRIDE * Self::MAX_HEIGHT as usize],
            rooms: Vec::new(),
        }
    }
}

pub struct World {
    pub asset: super::DataAsset,
    pub regions: Vec<WorldRegion>,
}

impl World {
    pub fn new(id: DataAssetId, name: String) -> Self {
        World {
            asset: super::DataAsset::new(super::DataAssetType::World, id, name),
            regions: Vec::new(),
        }
    }
}

impl super::HashedDataAsset for World {
    fn data_hash(&self, hasher: &mut super::DataAssetHasher) {
        hasher.add_data_asset(&self.asset);
        for region in &self.regions {
            hasher.add_str(&region.name);
            hasher.add_u8(region.x);
            hasher.add_u8(region.y);
            hasher.add_u8(region.width);
            hasher.add_u8(region.height);
            for block in &region.blocks {
                match block {
                    Some(n) => { hasher.add_u16(*n as u16); }
                    None    => { hasher.add_u16(0xff00); }
                }
            }
        }
    }
}

impl super::DuplicableAsset<World> for World {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        World {
            asset: self.asset.duplicate(dup_id, dup_name),
            regions: self.regions.clone(),
        }
    }
}

impl super::GenericAsset for World {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: num_regions(2) + pad(2) + regions<ptr>(4)
        let header = 2 + 2 + 4;

        // region[0..num_regions]: x(1) + y(1) + w(1) + h(1) + block_bitmap<ptr>(4) + blocks<ptr>(4) + room_ids<ptr>(4)
        let regions = self.regions.len() * (1 + 1 + 1 + 1 + 4 + 4 + 4);

        // room ids: regions[..]: 4*ceil((w*h)/32) + region.num_set_blocks + region.num_rooms * room_index(2)
        let room_ids = self.regions.iter().fold(0, |sum, region| {
            let mut num_set_blocks = 0usize;
            for y in 0..region.height {
                for x in 0..region.width {
                    if region.blocks[y as usize * WorldRegion::BLOCK_STRIDE + x as usize].is_some() {
                        num_set_blocks += 1;
                    }
                }
            }
            let width = region.width as usize;
            let height = region.height as usize;
            sum + num_set_blocks + 4 * (width * height).div_ceil(32) + region.rooms.len() * 2
        });

        header + regions + room_ids
    }
}
