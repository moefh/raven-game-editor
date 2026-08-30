#[derive(std::hash::Hash)]
pub struct MapData {
    pub asset: super::DataAsset,
    pub tileset_id: super::DataAssetId,
    pub tile_anim_id: Option<super::DataAssetId>,
    pub width: u32,
    pub height: u32,
    pub para_width: u32,
    pub para_height: u32,
    pub fg_tiles: Vec<u8>,
    pub bg_tiles: Vec<u8>,
    pub fx_tiles: Vec<u8>,
    pub para_tiles: Vec<u8>,
}

impl MapData {
    pub const NO_TILE: u8 = 0xff;

    pub fn new(id: super::DataAssetId, name: String, tileset_id: super::DataAssetId) -> Self {
        let width = 24;
        let height = 18;
        let para_width = 0;
        let para_height = 0;
        MapData {
            asset: super::DataAsset::new(super::DataAssetType::MapData, id, name),
            tileset_id,
            tile_anim_id: None,
            width,
            height,
            para_width,
            para_height,
            fg_tiles: vec![Self::NO_TILE; (width * height) as usize],
            bg_tiles: vec![0; (width * height) as usize],
            fx_tiles: vec![Self::NO_TILE; (width * height) as usize],
            para_tiles: vec![Self::NO_TILE; (para_width * para_height) as usize],
        }
    }
}

impl super::DataHashAsset for MapData {
    fn data_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;

        self.asset.asset_type.hash(state);
        self.asset.name.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.para_width.hash(state);
        self.para_height.hash(state);
        self.fg_tiles.hash(state);
        self.bg_tiles.hash(state);
        self.fx_tiles.hash(state);
        self.para_tiles.hash(state);
    }
}

impl super::DuplicableAsset<MapData> for MapData {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        MapData {
            asset: self.asset.duplicate(dup_id, dup_name),
            tileset_id: self.tileset_id,
            tile_anim_id: self.tile_anim_id,
            width: self.width,
            height: self.height,
            para_width: self.para_width,
            para_height: self.para_height,
            fg_tiles: self.fg_tiles.clone(),
            bg_tiles: self.bg_tiles.clone(),
            fx_tiles: self.fx_tiles.clone(),
            para_tiles: self.para_tiles.clone(),
        }
    }
}

impl super::GenericAsset for MapData {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: u16(2) * (w,h,para_w,para_h) + ptr(4) * (tileset,tile_anim,tile_data)<ptr>
        let header = 2usize * 4usize + 4usize * 3usize;

        // tile_data:
        //   tiles: w * h * [u8(1) * (fg,bg,fx)]
        //   parallax: para_w * para_h * [u8(1) * (para)]
        let full_tiles = (self.width as usize) * (self.height as usize) * 3usize;
        let para_tiles = (self.para_width as usize) * (self.para_height as usize);

        header + full_tiles + para_tiles
    }
}
