#[derive(std::hash::Hash)]
pub struct Tileset {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub data: Vec<u8>,
}

pub struct CreationData {
    pub width: u32,
    pub height: u32,
    pub num_tiles: u32,
    pub pixels: Vec<u8>,
}

impl Tileset {
    pub const TILE_SIZE: u32 = 16;
    pub const EMPTY_COLOR: u8 = 0xff;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let num_tiles = 10;
        Tileset {
            asset: super::DataAsset::new(super::DataAssetType::Tileset, id, name),
            width: Self::TILE_SIZE,
            height: Self::TILE_SIZE,
            num_tiles,
            data: vec![Self::EMPTY_COLOR; (Self::TILE_SIZE*Self::TILE_SIZE*num_tiles) as usize],
        }
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        Tileset {
            asset: super::DataAsset::new(super::DataAssetType::Tileset, id, name),
            width: data.width,
            height: data.height,
            num_tiles: data.num_tiles,
            data: data.pixels,
        }
    }
}

impl super::DuplicableAsset<Tileset> for Tileset {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        Tileset {
            asset: self.asset.duplicate(dup_id, dup_name),
            width: self.width,
            height: self.height,
            num_tiles: self.num_tiles,
            data: self.data.clone(),
        }
    }
}

impl super::GenericAsset for Tileset {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: w(4) + h(4) + stride(4) + num_tiles(4) + data<ptr>(4)
        let header = 4usize * 5usize;

        // image: (4*stride) * height * num_tiles
        let image = (4 * self.width.div_ceil(4) * self.height * self.num_tiles) as usize;

        header + image
    }
}
