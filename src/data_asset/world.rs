use super::DataAssetId;

#[derive(Clone, std::hash::Hash)]
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

#[derive(std::hash::Hash)]
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
        // TODO
        0
    }
}
