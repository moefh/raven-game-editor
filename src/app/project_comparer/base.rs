use crate::data_store::{
    DataStore,
    DataStoreItemTable,
    DataStoreEffectTable,
};

pub struct ItemTableDiff {
    pub diffs: Vec<usize>,
    pub cur_only: core::range::Range<usize>,
    pub other_only: core::range::Range<usize>,
}

impl ItemTableDiff {
    fn new() -> Self {
        ItemTableDiff {
            diffs: Vec::new(),
            cur_only: core::range::Range::from(0..0),
            other_only: core::range::Range::from(0..0),
        }
    }

    fn clear(&mut self) {
        self.diffs.clear();
        self.cur_only = core::range::Range::from(0..0);
        self.other_only = core::range::Range::from(0..0);
    }

    fn compare(
        &mut self,
        cur_table: &DataStoreItemTable,
        other_table: &DataStoreItemTable,
        cur: &DataStore,
        other: &DataStore
    ) {
        self.clear();
        for (index, (cur_item, other_item)) in cur_table.items.iter().zip(other_table.items.iter()).enumerate() {
            if cur_item.name != other_item.name {
                self.diffs.push(index);
            } else {
                let cur_s = cur.assets.sprites.get(&cur_item.sprite_id);
                let other_s = other.assets.sprites.get(&other_item.sprite_id);
                if let Some(cur_s) = cur_s && let Some(other_s) = other_s && cur_s.asset.name != other_s.asset.name {
                    self.diffs.push(index);
                }
            }
        }
        if cur_table.items.len() > other_table.items.len() {
            self.cur_only = core::range::Range::from(other_table.items.len()..cur_table.items.len());
        } else if other_table.items.len() > cur_table.items.len() {
            self.other_only = core::range::Range::from(cur_table.items.len()..other_table.items.len());
        }
    }

    pub fn has_differences(&self) -> bool {
        ! (self.diffs.is_empty() && self.cur_only.is_empty() && self.other_only.is_empty())
    }
}

pub struct EffectTableDiff {
    pub diffs: Vec<usize>,
    pub cur_only: core::range::Range<usize>,
    pub other_only: core::range::Range<usize>,
}

impl EffectTableDiff {
    fn new() -> Self {
        EffectTableDiff {
            diffs: Vec::new(),
            cur_only: core::range::Range::from(0..0),
            other_only: core::range::Range::from(0..0),
        }
    }

    fn clear(&mut self) {
        self.diffs.clear();
        self.cur_only = core::range::Range::from(0..0);
        self.other_only = core::range::Range::from(0..0);
    }

    fn compare(&mut self, cur_table: &DataStoreEffectTable, other_table: &DataStoreEffectTable) {
        self.clear();
        for (index, (cur_name, other_name)) in cur_table.names.iter().zip(other_table.names.iter()).enumerate() {
            if cur_name != other_name {
                self.diffs.push(index);
            }
        }
        if cur_table.names.len() > other_table.names.len() {
            self.cur_only = core::range::Range::from(other_table.names.len()..cur_table.names.len());
        } else if other_table.names.len() > cur_table.names.len() {
            self.other_only = core::range::Range::from(cur_table.names.len()..other_table.names.len());
        }
    }

    pub fn has_differences(&self) -> bool {
        ! (self.diffs.is_empty() && self.cur_only.is_empty() && self.other_only.is_empty())
    }
}

pub struct BaseProjectDiff {
    pub cur_data_size: usize,
    pub other_data_size: usize,
    pub upgrade_table_diff: ItemTableDiff,
    pub collectable_table_diff: ItemTableDiff,
    pub pickup_table_diff: ItemTableDiff,
    pub effect_table_diff: EffectTableDiff,
    pub flag_table_diff: EffectTableDiff,
}

impl BaseProjectDiff {
    pub fn new() -> Self {
        BaseProjectDiff {
            cur_data_size: 0,
            other_data_size: 0,
            upgrade_table_diff: ItemTableDiff::new(),
            collectable_table_diff: ItemTableDiff::new(),
            pickup_table_diff: ItemTableDiff::new(),
            effect_table_diff: EffectTableDiff::new(),
            flag_table_diff: EffectTableDiff::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cur_data_size = 0;
        self.other_data_size = 0;
        self.upgrade_table_diff.clear();
        self.collectable_table_diff.clear();
        self.pickup_table_diff.clear();
        self.effect_table_diff.clear();
        self.flag_table_diff.clear();
    }

    pub fn compare(&mut self, store: &DataStore, other: &DataStore) {
        self.cur_data_size = store.data_size();
        self.other_data_size = other.data_size();
        self.upgrade_table_diff.compare(&store.tables.upgrade, &other.tables.upgrade, store, other);
        self.collectable_table_diff.compare(&store.tables.collectable, &other.tables.collectable, store, other);
        self.pickup_table_diff.compare(&store.tables.pickup, &other.tables.pickup, store, other);
        self.effect_table_diff.compare(&store.tables.effect, &other.tables.effect);
        self.flag_table_diff.compare(&store.tables.flag, &other.tables.flag);
    }

    pub fn has_table_differences(&self) -> bool {
        self.upgrade_table_diff.has_differences() ||
            self.pickup_table_diff.has_differences() ||
            self.collectable_table_diff.has_differences() ||
            self.effect_table_diff.has_differences() ||
            self.flag_table_diff.has_differences()
    }

}
