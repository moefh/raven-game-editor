
use crate::data_asset::{
    calc_asset_data_hash,
    DataAssetStore,
    DataAssetId,
    MapData,
};

pub struct MapListDiff {
    pub diffs: Vec<MapDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl MapListDiff {
    pub fn new() -> Self {
        MapListDiff {
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
        self.diffs.clear();
        self.cur_only.clear();
        self.other_only.clear();

        self.cur_only.splice(.., cur_store.asset_ids.maps.iter().copied());
        for other in other_store.assets.maps.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.maps.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_map_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = MapDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct MapDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
}

impl MapDiff {
    pub fn compare(cur: &MapData, other: &MapData) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(MapDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
