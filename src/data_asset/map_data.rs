#[derive(Clone)]
pub struct MapTileChange {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Option<u8>>,
}

impl MapTileChange {
    pub fn new(name: impl Into<String>, x: u32, y: u32, width: u32, height: u32) -> Self {
        MapTileChange {
            name: name.into(),
            x,
            y,
            width,
            height,
            tiles: vec![None; (width * height) as usize],
        }
    }
}

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
    pub tile_changes: Vec<MapTileChange>,
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
            tile_changes: Vec::new(),
        }
    }
}

impl super::HashedDataAsset for MapData {
    fn data_hash(&self, hasher: &mut super::DataAssetHasher) {
        hasher.add_data_asset(&self.asset);
        hasher.add_asset_id(self.tileset_id);
        match self.tile_anim_id {
            Some(id) => { hasher.add_u8(1); hasher.add_asset_id(id); }
            None => { hasher.add_u8(0); }
        }
        hasher.add_u32(self.width);
        hasher.add_u32(self.height);
        hasher.add_bool(self.has_bb);
        hasher.add_bytes(&self.fg_tiles);
        hasher.add_bytes(&self.bg_tiles);
        hasher.add_bytes(&self.fx_tiles);
        hasher.add_bytes(&self.bb_tiles);
        for change in &self.tile_changes {
            hasher.add_str(&change.name);
            hasher.add_u32(change.x);
            hasher.add_u32(change.y);
            hasher.add_u32(change.width);
            hasher.add_u32(change.height);
            for tile in &change.tiles {
                match tile {
                    Some(n) => { hasher.add_u16(*n as u16); }
                    None    => { hasher.add_u16(0xff00); }
                }
            }
        }
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
            tile_changes: self.tile_changes.clone(),
        }
    }
}

impl super::GenericAsset for MapData {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: u16(2) * (w,h) + ptr(4) * (tileset,tile_anim,tile_data)<ptr> + num_tile_changes(2) + pad(2) + tile_changes<ptr>(4)
        let header = 2usize * 2 + 4usize * 3 + 2usize + 2usize + 4usize;

        // tile_data:
        //   tiles:    w * h * [u8(1) * (fg,bg,fx)]
        //   bb_tiles: w * h * [u8(1) * (bb)]
        let tiles = (self.width as usize) * (self.height as usize) * 3;
        let bb_tiles = if self.has_bb { (self.width as usize) * (self.height as usize) } else { 0 };

        // tile changes:
        //   header: u116(2) * (x,y,w,h) + ptr(4)
        //   changes[..]: 4*ceil((w*h)/32) + change.num_set_blocks
        let tile_changes = self.tile_changes.len() * (2usize * 4 + 4usize);
        let tile_change_tiles = self.tile_changes.iter().fold(0, |sum, change| {
            let mut num_set_tiles = 0;
            for y in 0..change.height {
                for x in 0..change.width {
                    if change.tiles[(y * change.width + x) as usize].is_some() {
                        num_set_tiles += 1;
                    }
                }
            }
            let width = change.width as usize;
            let height = change.height as usize;
            sum + num_set_tiles + 4 * (width * height).div_ceil(32)
        });

        header + tiles + bb_tiles + tile_changes + tile_change_tiles
    }
}
