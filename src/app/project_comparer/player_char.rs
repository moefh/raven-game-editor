
use crate::data_store::{
    calc_asset_data_hash,
    DataStore,
    DataAssetId,
    PlayerChar,
};

pub struct PlayerCharListDiff {
    pub diffs: Vec<PlayerCharDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl PlayerCharListDiff {
    pub fn new() -> Self {
        PlayerCharListDiff {
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

        self.cur_only.splice(.., cur_store.asset_ids.player_chars.iter().copied());
        for other in other_store.assets.player_chars.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.player_chars.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_player_char_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = PlayerCharDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct PlayerCharDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl PlayerCharDiff {
    pub fn compare(cur: &PlayerChar, other: &PlayerChar) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(PlayerCharDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
