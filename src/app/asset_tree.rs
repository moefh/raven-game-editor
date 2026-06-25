use std::collections::HashMap;

use crate::data_asset::{DataAssetType, DataAssetId, DataAssetStore, DataAsset, GenericAsset};
use crate::misc::asset_defs::ASSET_DEFS;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssetTreeNodeId {
    id: u32,
}

struct AssetTreeNodeIdGenerator {
    next: u32,
}

impl AssetTreeNodeIdGenerator {
    pub fn new() -> Self {
        AssetTreeNodeIdGenerator {
            next: 0,
        }
    }

    fn generate_id(&mut self) -> AssetTreeNodeId {
        let id = self.next;
        self.next += 1;
        AssetTreeNodeId {
            id,
        }
    }
}

pub struct AssetTreeItem {
    pub id: DataAssetId,
    pub name: String,
    used: bool,
}

impl AssetTreeItem {
    fn new(asset: &DataAsset, name: String) -> Self {
        AssetTreeItem {
            id: asset.id,
            name,
            used: true,
        }
    }

    fn mark_used(&mut self) {
        self.used = true;
    }

    fn clear_used_mark(&mut self) {
        self.used = false;
    }
}

pub struct AssetTreeContainer {
    pub node_id: AssetTreeNodeId,
    pub name: String,
    pub level: usize,
    containers: Vec<AssetTreeContainer>,
    assets: Vec<AssetTreeItem>,
    used: bool,
}

impl AssetTreeContainer {
    fn new(node_id: AssetTreeNodeId, name: String, level: usize) -> Self {
        AssetTreeContainer {
            node_id,
            name,
            level,
            used: true,
            containers: Vec::new(),
            assets: Vec::new(),
        }
    }

    fn sort(&mut self) {
        self.assets.sort_by(|a, b| { a.name.cmp(&b.name) });
        self.containers.sort_by(|a, b| { a.name.cmp(&b.name) });
        for tree in &mut self.containers {
            tree.sort();
        }
    }

    fn clear_used_mark(&mut self) {
        self.used = false;
        for child in self.containers.iter_mut() {
            child.clear_used_mark();
        }
        for asset in self.assets.iter_mut() {
            asset.clear_used_mark();
        }
    }

    fn mark_used(&mut self) {
        self.used = true;
    }

    fn remove_unused(&mut self) {
        self.assets.retain(|a| a.used);
        self.containers.retain(|c| c.used || c.level == 0);
        for child in &mut self.containers {
            child.remove_unused();
        }
    }

    fn insert_asset(&mut self, asset: &DataAsset, name: &str) {
        match self.assets.iter_mut().find(|a| a.id == asset.id) {
            Some(a) => {
                if a.name != name {
                    a.name = name.to_owned();
                }
                a.mark_used();
            }
            None => {
                self.assets.push(AssetTreeItem::new(asset, name.to_owned()));
            }
        }
        self.mark_used();
    }

    fn insert(&mut self, asset: &DataAsset, name_parts: &[&str], id_generator: &mut AssetTreeNodeIdGenerator) {
        match name_parts {
            &[] => {}

            &[name] => {
                self.insert_asset(asset, name);
            }

            name_parts => {
                let child = match self.containers.iter_mut().find(|t| t.name == name_parts[0]) {
                    Some(child) => {
                        child
                    }
                    None => {
                        let child = AssetTreeContainer::new(id_generator.generate_id(),
                                                            name_parts[0].to_owned(), self.level + 1);
                        self.containers.push_mut(child)
                    }
                };
                child.insert(asset, &name_parts[1..], id_generator);
                self.mark_used();
            }
        }
    }

    fn update_asset(&mut self, asset: &DataAsset, id_generator: &mut AssetTreeNodeIdGenerator) {
        if ! asset.name.contains(DataAsset::PATH_SEPARATOR) {  // avoid splitting the string if we can
            self.insert_asset(asset, &asset.name);
        } else {
            let name_parts: Vec<&str> = asset.name.split(DataAsset::PATH_SEPARATOR).collect();
            self.insert(asset, &name_parts, id_generator);
        }
    }

    fn get_node_name_parts(&self, node_id: AssetTreeNodeId, parts: &mut Vec<String>) -> bool {
        let found = if node_id == self.node_id {
            true
        } else {
            self.containers.iter().position(|c| c.get_node_name_parts(node_id, parts)).is_some()
        };
        if found && self.level != 0 {
            parts.push(self.name.clone());
        }
        found
    }

    fn get_node_name(&self, node_id: AssetTreeNodeId) -> Option<String> {
        let mut parts = Vec::new();
        if self.get_node_name_parts(node_id, &mut parts) && ! parts.is_empty() {
            Some(parts.into_iter().rev().collect::<Vec<String>>().join(DataAsset::PATH_SEPARATOR))
        } else {
            None
        }
    }

    pub fn show_inside(&self, id_prefix: &str, ui: &mut egui::Ui, open: bool,
                       show_folder: &mut impl FnMut(&mut egui::Ui, &AssetTreeContainer) -> egui::Response,
                       show_item: &mut impl FnMut(&mut egui::Ui, &AssetTreeContainer, &AssetTreeItem)) {
        let tree_node_id = ui.make_persistent_id(format!("{}_{}", id_prefix, self.node_id.id));
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, open);
        let mut toggle_node_open = false;
        let mut header_resp = node.show_header(ui, |ui| {
            toggle_node_open = show_folder(ui, self).clicked();
        });
        if toggle_node_open {
            header_resp.toggle();
        }
        header_resp.body(|ui| {
            for tree in &self.containers {
                tree.show_inside(id_prefix, ui, self.assets.is_empty(), show_folder, show_item);
            }
            for asset_node in &self.assets {
                show_item(ui, self, asset_node);
            }
        });
    }
}

pub struct SimpleAssetTree {
    root: AssetTreeContainer,
    id_generator: AssetTreeNodeIdGenerator,
    id_prefix: String,
}

impl SimpleAssetTree {
    pub fn new(id_prefix: impl Into<String>, name: impl Into<String>) -> Self {
        let mut id_generator = AssetTreeNodeIdGenerator::new();
        let root = AssetTreeContainer::new(id_generator.generate_id(), name.into(), 0);
        SimpleAssetTree {
            root,
            id_generator,
            id_prefix: id_prefix.into(),
        }
    }

    pub fn from_assets<'a, T>(id_prefix: impl Into<String>, name: impl Into<String>,
                              assets: impl Iterator<Item = &'a T>) -> Self
    where T: GenericAsset + 'a
    {
        let mut tree = Self::new(id_prefix, name);
        for asset in assets {
            tree.root.update_asset(asset.asset(), &mut tree.id_generator);
        }
        tree
    }

    pub fn update_assets<'a, T: GenericAsset + 'a>(&mut self, assets: impl Iterator<Item = &'a T>) {
        self.root.clear_used_mark();

        // insert new assets and mark existing assets used
        for asset in assets {
            self.root.update_asset(asset.asset(), &mut self.id_generator);
        }

        // remove childless containers and orphan asset nodes
        self.root.remove_unused();

        self.root.sort();
    }

    pub fn show_inside(&self, ui: &mut egui::Ui, open: bool,
                       show_folder: &mut impl FnMut(&mut egui::Ui, &AssetTreeContainer) -> egui::Response,
                       show_item: &mut impl FnMut(&mut egui::Ui, &AssetTreeContainer, &AssetTreeItem)) {
        self.root.show_inside(&self.id_prefix, ui, open, show_folder, show_item);
    }

    //pub fn get_node_name(&self, node_id: AssetTreeNodeId) -> Option<String> {
    //    self.root.get_node_name(node_id)
    //}
}

pub struct StoreAssetTree {
    roots: HashMap<DataAssetType, AssetTreeContainer>,
    id_generator: AssetTreeNodeIdGenerator,
}

impl StoreAssetTree {
    pub fn new() -> Self {
        let mut id_generator = AssetTreeNodeIdGenerator::new();
        let mut roots = HashMap::new();
        for def in ASSET_DEFS {
            let tree = AssetTreeContainer::new(id_generator.generate_id(), def.tree_root_item.to_owned(), 0);
            roots.insert(def.asset_type, tree);
        }
        StoreAssetTree {
            roots,
            id_generator,
        }
    }

    pub fn update(&mut self, store: &DataAssetStore) {
        for tree in self.roots.values_mut() {
            tree.clear_used_mark();
        }

        // insert new assets and mark existing assets used
        for (asset_type, tree) in self.roots.iter_mut() {
            for &asset_id in store.asset_ids.ids_of_type(*asset_type) {
                if let Some(asset) = store.assets.get_asset(asset_id) {
                    tree.update_asset(asset, &mut self.id_generator);
                }
            }
        }

        // remove childless containers and orphan asset nodes
        for tree in self.roots.values_mut() {
            tree.remove_unused();
        }

        // sort containers and assets
        for tree in self.roots.values_mut() {
            tree.sort();
        }
    }

    pub fn get_tree_of_type(&self, asset_type: DataAssetType) -> Option<&AssetTreeContainer> {
        self.roots.get(&asset_type)
    }

    pub fn get_node_name(&self, asset_type: DataAssetType, node_id: AssetTreeNodeId) -> Option<String> {
        self.roots.get(&asset_type).and_then(|tree| tree.get_node_name(node_id))
    }
}
