
use crate::data_asset::{
    calc_asset_data_hash,
    DataAssetStore,
    DataAssetId,
    TileAnimation,
};

pub struct TileAnimationListDiff {
    pub diffs: Vec<TileAnimationDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl TileAnimationListDiff {
    pub fn new() -> Self {
        TileAnimationListDiff {
            diffs: Vec::new(),
            cur_only: Vec::new(),
            other_only: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diffs.is_empty() && self.cur_only.is_empty() && self.other_only.is_empty()
    }

    pub fn clear(&mut self) {
        self.diffs.clear();
        self.cur_only.clear();
        self.other_only.clear();
    }

    pub fn compare(&mut self, cur_store: &DataAssetStore, other_store: &DataAssetStore) {
        self.clear();

        self.cur_only.splice(.., cur_store.asset_ids.tile_anims.iter().copied());
        for other in other_store.assets.tile_anims.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.tile_anims.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_tile_animation_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = TileAnimationDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct TileAnimationDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl TileAnimationDiff {
    pub fn compare(cur: &TileAnimation, other: &TileAnimation) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(TileAnimationDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
