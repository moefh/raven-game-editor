#[derive(std::hash::Hash)]
pub struct MapData {
    pub asset: super::DataAsset,
    pub tileset_id: super::DataAssetId,
    pub tile_anim_id: Option<super::DataAssetId>,
    pub width: u32,
    pub height: u32,
    pub has_bb: bool,
    pub fg_tiles: Vec<u8>,
    pub bg_tiles: Vec<u8>,
    pub fx_tiles: Vec<u8>,
    pub bb_tiles: Vec<u8>,
}

impl MapData {
    pub const NO_TILE: u8 = 0xff;

    pub fn new(id: super::DataAssetId, name: String, tileset_id: super::DataAssetId) -> Self {
        let width = 24;
        let height = 18;
        MapData {
            asset: super::DataAsset::new(super::DataAssetType::MapData, id, name),
            tileset_id,
            tile_anim_id: None,
            width,
            height,
            has_bb: false,
            fg_tiles: vec![Self::NO_TILE; (width * height) as usize],
            bg_tiles: vec![0; (width * height) as usize],
            fx_tiles: vec![Self::NO_TILE; (width * height) as usize],
            bb_tiles: Vec::new(),
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
        self.has_bb.hash(state);
        self.fg_tiles.hash(state);
        self.bg_tiles.hash(state);
        self.fx_tiles.hash(state);
        self.bb_tiles.hash(state);
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
            has_bb: self.has_bb,
            fg_tiles: self.fg_tiles.clone(),
            bg_tiles: self.bg_tiles.clone(),
            fx_tiles: self.fx_tiles.clone(),
            bb_tiles: self.bb_tiles.clone(),
        }
    }
}

impl super::GenericAsset for MapData {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: u16(2) * (w,h) + ptr(4) * (tileset,tile_anim,tile_data)<ptr>
        let header = 2usize * 2usize + 4usize * 3usize;

        // tile_data:
        //   tiles:    w * h * [u8(1) * (fg,bg,fx,bb)]
        //   bb_tiles: w * h * [u8(1) * (bb)]
        let tiles = (self.width as usize) * (self.height as usize) * 3usize;
        let bb_tiles = if self.has_bb { (self.width as usize) * (self.height as usize) } else { 0 };

        header + tiles + bb_tiles
    }
}
