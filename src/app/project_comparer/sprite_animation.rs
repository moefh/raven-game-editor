
use crate::data_asset::{
    calc_asset_data_hash,
    DataAssetStore,
    DataAssetId,
    SpriteAnimation,
};

pub struct SpriteAnimationListDiff {
    pub diffs: Vec<SpriteAnimationDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl SpriteAnimationListDiff {
    pub fn new() -> Self {
        SpriteAnimationListDiff {
            diffs: Vec::new(),
            cur_only: Vec::new(),
            other_only: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diffs.is_empty() && self.cur_only.is_empty() && self.other_only.is_empty()
    }

    pub fn compare(&mut self, cur_store: &DataAssetStore, other_store: &DataAssetStore) {
        self.diffs.clear();
        self.cur_only.clear();
        self.other_only.clear();

        self.cur_only.splice(.., cur_store.asset_ids.animations.iter().copied());
        for other in other_store.assets.animations.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.animations.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_sprite_animation_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = SpriteAnimationDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct SpriteAnimationDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl SpriteAnimationDiff {
    pub fn compare(cur: &SpriteAnimation, other: &SpriteAnimation) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(SpriteAnimationDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
