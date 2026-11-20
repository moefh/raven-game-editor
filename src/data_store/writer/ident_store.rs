use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Result, Error};

use super::{
    DataStore,
    DataAssetId,
    DataAssetType,
    DataAsset,
};
use super::super::utils::{
    get_asset_type_upper_name,
};

pub struct IdentStore {
    pub prefix_upper: String,
    pub prefix_lower: String,
    pub type_to_id_to_name_id: HashMap<DataAssetType, HashMap<DataAssetId, String>>,
    pub type_to_id_to_index: HashMap<DataAssetType, HashMap<DataAssetId, usize>>,
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
            type_to_id_to_name_id: HashMap::new(),
            type_to_id_to_index: HashMap::new(),
        }
    }

    pub fn lower_cleanup(name: &str) -> String {
        let mut clean = Self::cleanup(name);
        clean.make_ascii_lowercase();
        clean
    }

    pub fn upper_cleanup(name: &str) -> String {
        let mut clean = Self::cleanup(name);
        clean.make_ascii_uppercase();
        clean
    }

    pub fn cleanup(name: &str) -> String {
        let mut clean = String::with_capacity(name.len());

        for ch in name.chars() {
            if matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                clean.push(ch);
            } else {
                clean.push('_');
            }
        }
        clean
    }

    pub fn add_unique_name_id<K: Eq + Hash + Copy>(key: K, name_id: &str, namespace: &mut HashMap<K, String>) {
        let base = Self::lower_cleanup(name_id);
        let mut new_name_id = base.clone();
        let mut num = 1;
        loop {
            if ! namespace.values().any(|s| *s == new_name_id) {
                namespace.insert(key, new_name_id.clone());
                return;
            }
            new_name_id.clear();
            new_name_id.push_str(&base);
            new_name_id.push_str(&num.to_string());
            num += 1;
        }
    }

    pub fn get_asset_name_id(&self, asset_type: DataAssetType, id: DataAssetId) -> Result<&String> {
        self.type_to_id_to_name_id.get(&asset_type).and_then(|id_to_name_id| id_to_name_id.get(&id)).ok_or_else(|| {
            Error::other(format!("can't find asset id {} of type {:?}", id, asset_type))
        })
    }

    pub fn get_asset_ident(&self, asset_type: DataAssetType, id: DataAssetId) -> Result<String> {
        let name_id = self.get_asset_name_id(asset_type, id)?;
        let name_id_upper = name_id.to_ascii_uppercase();
        let mut ident = String::new();
        ident.push_str(&self.prefix_upper);
        ident.push('_');
        ident.push_str(get_asset_type_upper_name(asset_type));
        ident.push_str("_ID_");
        ident.push_str(&name_id_upper);
        Ok(ident)
    }

    pub fn get_asset_ident_or_none(&self, asset_type: DataAssetType, id: Option<DataAssetId>) -> Result<String> {
        if let Some(id) = id {
            self.get_asset_ident(asset_type, id)
        } else {
            Ok("0xffff".to_owned())
        }
    }

    pub fn add_assets(&mut self, asset_type: DataAssetType, store: &DataStore) {
        self.type_to_id_to_index.entry(asset_type).or_insert_with(|| {
            let mut id_to_index = HashMap::<DataAssetId, usize>::new();

            for (index, &id) in store.asset_ids.ids_of_type(asset_type).enumerate() {
                id_to_index.insert(id, index);
            }

            id_to_index
        });

        self.type_to_id_to_name_id.entry(asset_type).or_insert_with(|| {
            let mut id_to_name_id = HashMap::<DataAssetId, String>::new();

            for &id in store.asset_ids.ids_of_type(asset_type) {
                if let Some(asset) = store.assets.get_asset(id) {
                    IdentStore::add_unique_name_id(id, &DataAsset::name_to_identifier(&asset.name), &mut id_to_name_id);
                }
            }

            id_to_name_id
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
