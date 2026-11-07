
use crate::data_store::{
    calc_asset_data_hash,
    DataStore,
    DataAssetId,
    Enemy,
};

pub struct EnemyListDiff {
    pub diffs: Vec<EnemyDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl EnemyListDiff {
    pub fn new() -> Self {
        EnemyListDiff {
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

    pub fn compare(&mut self, cur_store: &DataStore, other_store: &DataStore) {
        self.clear();

        self.cur_only.splice(.., cur_store.asset_ids.enemies.iter().copied());
        for other in other_store.assets.enemies.iter() {
            self.cur_only.retain(|cur_id| {
                ! cur_store.assets.enemies.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name)
            });
            if let Some(cur) = super::get_enemy_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = EnemyDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct EnemyDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl EnemyDiff {
    pub fn compare(cur: &Enemy, other: &Enemy) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(EnemyDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
