use crate::data_store::{
    calc_asset_data_hash,
    DataStore,
    DataAssetId,
    Tileset,
};

use super::ImageDiff;

pub struct TilesetListDiff {
    pub diffs: Vec<TilesetDiff>,
    pub other_only: Vec<DataAssetId>,
    pub cur_only: Vec<DataAssetId>,
}

impl TilesetListDiff {
    pub fn new() -> Self {
        TilesetListDiff {
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

        self.cur_only.splice(.., cur_store.asset_ids.tilesets.iter().copied());
        for other in other_store.assets.tilesets.iter() {
            self.cur_only.retain(|cur_id| ! cur_store.assets.tilesets.get(cur_id).is_some_and(|cur| cur.asset.name == other.asset.name));
            if let Some(cur) = super::get_tileset_by_name(cur_store, &other.asset.name) {
                if let Some(diff) = TilesetDiff::compare(cur, other) {
                    self.diffs.push(diff);
                }
            } else {
                self.other_only.push(other.asset.id);
            }
        }
    }
}

pub struct TilesetDiff {
    pub cur_id: DataAssetId,
    pub other_id: DataAssetId,
    pub tiles_diff: Vec<ImageDiff>,
}

impl TilesetDiff {
    pub fn compare(cur: &Tileset, other: &Tileset) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(TilesetDiff {
                cur_id: cur.asset.id,
                other_id: other.asset.id,
                tiles_diff: ImageDiff::compare(cur, other),
            })
        } else {
            None
        }
    }
}
