use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Result, Error};

use super::{DataAssetStore, DataAssetId, DataAssetType};

pub struct IdentStore {
    pub prefix_upper: String,
    pub prefix_lower: String,
    pub type_names: HashMap<DataAssetType, HashMap<DataAssetId, String>>,
    pub type_indices: HashMap<DataAssetType, HashMap<DataAssetId, usize>>,
}

impl IdentStore {
    pub fn new(prefix: &str) -> Self {
        let mut prefix_upper = Self::cleanup(prefix);
        let mut prefix_lower = prefix_upper.clone();
        prefix_upper.make_ascii_uppercase();
        prefix_lower.make_ascii_lowercase();

        IdentStore {
            prefix_upper,
            prefix_lower,
            type_names: HashMap::new(),
            type_indices: HashMap::new(),
        }
    }

    pub fn lower_cleanup(name: &str) -> String {
        let mut clean = Self::cleanup(name);
        clean.make_ascii_lowercase();
        clean
    }

    pub fn cleanup(name: &str) -> String {
        let mut clean = String::new();

        for ch in name.chars() {
            if matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                clean.push(ch);
            } else {
                clean.push('_');
            }
        }
        clean
    }

    pub fn add_unique_name<K: Eq + Hash + Copy>(key: K, name: &str, namespace: &mut HashMap<K, String>) {
        let base = Self::lower_cleanup(name);
        let mut new_name = base.clone();
        let mut num = 1;
        loop {
            if ! namespace.values().any(|s| *s == new_name) {
                namespace.insert(key, new_name.clone());
                return;
            }
            new_name.clear();
            new_name.push_str(&base);
            new_name.push_str(&num.to_string());
            num += 1;
        }
    }

    pub fn get_asset_name(&self, asset_type: DataAssetType, id: DataAssetId) -> Result<&String> {
        self.type_names.get(&asset_type).and_then(|names| names.get(&id)).ok_or_else(|| {
            Error::other(format!("can't find asset id {} of type {:?}", id, asset_type))
        })
    }

    pub fn get_asset_index(&self, asset_type: DataAssetType, id: DataAssetId) -> Result<usize> {
        self.type_indices.get(&asset_type).and_then(|indices| indices.get(&id)).copied().ok_or_else(|| {
            Error::other(format!("can't find asset id {} of type {:?}", id, asset_type))
        })
    }

    pub fn add_assets(&mut self, asset_type: DataAssetType, store: &DataAssetStore) {
        self.type_indices.entry(asset_type).or_insert_with(|| {
            let mut id_indices = HashMap::<DataAssetId, usize>::new();

            for (index, &id) in store.asset_ids.ids_of_type(asset_type).enumerate() {
                id_indices.insert(id, index);
            }

            id_indices
        });

        self.type_names.entry(asset_type).or_insert_with(|| {
            let mut id_names = HashMap::<DataAssetId, String>::new();

            for &id in store.asset_ids.ids_of_type(asset_type) {
                if let Some(asset) = store.assets.get_asset(id) {
                    IdentStore::add_unique_name(id, &asset.name, &mut id_names);
                }
            }

            id_names
        });
    }

    /*
    pub fn upper(&self, name: &str) -> String {
        let mut ident = self.prefix_upper.clone();
        ident.push('_');
        ident.push_str(&Self::cleanup(name));
        ident.make_ascii_uppercase();
        ident
    }

    pub fn lower(&self, name: &str) -> String {
        let mut ident = self.prefix_lower.clone();
        ident.push('_');
        ident.push_str(&Self::cleanup(name));
        ident.make_ascii_lowercase();
        ident
    }
    */
}
