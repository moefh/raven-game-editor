
use crate::data_asset::{
    calc_asset_data_hash,
    DataAssetStore,
    DataAssetId,
    Tileset,
};

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

    pub fn compare(&mut self, cur_store: &DataAssetStore, other_store: &DataAssetStore) {
        self.diffs.clear();
        self.cur_only.clear();
        self.other_only.clear();

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
}

impl TilesetDiff {
    pub fn compare(cur: &Tileset, other: &Tileset) -> Option<Self> {
        let cur_hash = calc_asset_data_hash(cur);
        let other_hash = calc_asset_data_hash(other);
        if cur_hash != other_hash {
            Some(TilesetDiff { cur_id: cur.asset.id, other_id: other.asset.id })
        } else {
            None
        }
    }
}
