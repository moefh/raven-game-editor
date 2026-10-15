#[derive(Debug, Clone, Copy)]
pub struct TileAnimationLoop {
    pub start: u8,
    pub len: u8,
}

impl TileAnimationLoop {
    pub const EMPTY: Self = TileAnimationLoop { start: 0, len: 0 };
}

pub struct TileAnimation {
    pub asset: super::DataAsset,
    pub parent_tileset_id: super::DataAssetId,
    pub anim_tileset_id: super::DataAssetId,
    pub loops: [TileAnimationLoop; TileAnimation::NUM_LOOPS],
}

impl TileAnimation {
    pub const NUM_LOOPS: usize = 256;

    pub fn new(id: super::DataAssetId, name: String, parent_tileset_id: super::DataAssetId, anim_tileset_id: super::DataAssetId) -> Self {
        TileAnimation {
            asset: super::DataAsset::new(super::DataAssetType::TileAnimation, id, name),
            parent_tileset_id,
            anim_tileset_id,
            loops: [TileAnimationLoop::EMPTY; Self::NUM_LOOPS],
        }
    }
}

impl super::HashedDataAsset for TileAnimation {
    fn data_hash(&self, hasher: &mut super::DataAssetHasher) {
        hasher.add_data_asset(&self.asset);
        hasher.add_asset_id(self.parent_tileset_id);
        hasher.add_asset_id(self.anim_tileset_id);
        for aloop in &self.loops {
            hasher.add_u8(aloop.start);
            hasher.add_u8(aloop.len);
        }
    }
}

impl super::DuplicableAsset<TileAnimation> for TileAnimation {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        TileAnimation {
            asset: self.asset.duplicate(dup_id, dup_name),
            parent_tileset_id: self.parent_tileset_id,
            anim_tileset_id: self.anim_tileset_id,
            loops: self.loops,
        }
    }
}

impl super::GenericAsset for TileAnimation {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // loop: start(1) + length(1)
        let loop_size = 1usize + 1usize;

        // header: parent_tileset<ptr>(4) + anim_tileset<ptr>(4)
        let header =  4usize + 4usize;

        header + Self::NUM_LOOPS * loop_size
    }
}
