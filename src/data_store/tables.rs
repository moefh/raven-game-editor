use super::{
    DataAssetId,
    DataHasher,
};

pub struct DataStoreStringTableItem {
    pub name: String,
    pub value: String,
}

pub struct DataStoreStringTable {
    pub items: Vec<DataStoreStringTableItem>,
}

impl DataStoreStringTable {
    pub fn new() -> Self {
        DataStoreStringTable {
            items: Vec::new(),
        }
    }

    pub fn data_hash(&self, hasher: &mut DataHasher) {
        for item in &self.items {
            hasher.add_str(&item.name);
            hasher.add_str(&item.value);
        }
    }

    pub fn data_size(&self) -> usize {
        let pointers = 4 * self.items.len();
        let strings = self.items.iter().fold(0, |size, item| size + item.value.len() + 1);

        pointers + strings
    }
}

pub struct DataStoreEffectTable {
    pub names: Vec<String>,
}

impl DataStoreEffectTable {
    pub fn new() -> Self {
        DataStoreEffectTable {
            names: Vec::new(),
        }
    }

    pub fn data_hash(&self, hasher: &mut DataHasher) {
        for name in &self.names {
            hasher.add_str(name);
        }
    }
}

pub struct DataStoreTableItem {
    pub name: String,
    pub sprite_id: DataAssetId,
}

pub struct DataStoreItemTable {
    pub items: Vec<DataStoreTableItem>,
}

impl DataStoreItemTable {
    pub fn new() -> Self {
        DataStoreItemTable {
            items: Vec::new(),
        }
    }

    pub fn data_hash(&self, hasher: &mut DataHasher) {
        for item in &self.items {
            hasher.add_str(&item.name);
            hasher.add_asset_id(item.sprite_id);
        }
    }

    pub fn data_size(&self) -> usize {
        // ptr<4> * num_items
        4 * self.items.len()
    }
}
