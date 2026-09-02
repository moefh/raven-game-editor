
use crate::data_asset::{
    calc_asset_data_hash,
    DataAssetStore,
    DataAssetId,
    Sprite,
};

pub struct SpriteListDiff {
    pub diffs: Vec<SpriteDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl SpriteListDiff {
    pub fn new() -> Self {
        SpriteListDiff {
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

        self.cur_only.splice(.., cur_store.asset_ids.sprites.iter().copied());
        for other in other_store.assets.sprites.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.sprites.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_sprite_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = SpriteDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct SpriteDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl SpriteDiff {
    pub fn compare(cur: &Sprite, other: &Sprite) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(SpriteDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
