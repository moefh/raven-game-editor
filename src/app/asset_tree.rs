use std::collections::HashMap;

use crate::data_asset::{DataAssetType, DataAssetId, DataAssetStore, DataAsset, GenericAsset, AssetList};
use crate::misc::asset_defs::ASSET_DEFS;

pub const ASSET_NAME_SEPARATOR: &str = "__";

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssetTreeNodeId {
    id: u32,
}

struct AssetTreeNodeIdGenerator {
    next: u32,
}

impl AssetTreeNodeIdGenerator {
    fn new() -> Self {
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
    fn new(id: DataAssetId, name: String) -> Self {
        AssetTreeItem {
            id,
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

    pub fn from_assets<T: GenericAsset>(assets: &AssetList<T>, name: String) -> Self {
        let mut id_generator = AssetTreeNodeIdGenerator::new();
        let mut tree = AssetTreeContainer::new(id_generator.generate_id(), name, 0);
        for asset in assets.iter() {
            tree.update_asset(asset.asset(), &mut id_generator);
        }
        tree
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

    fn insert_asset(&mut self, asset_id: DataAssetId, name: &str) {
        match self.assets.iter_mut().find(|asset| asset.id == asset_id) {
            Some(asset) => {
                asset.mark_used();
            }
            None => {
                self.assets.push(AssetTreeItem::new(asset_id, name.to_owned()));
            }
        }
        self.mark_used();
    }

    fn insert(&mut self, asset_id: DataAssetId, name_parts: &[&str], id_generator: &mut AssetTreeNodeIdGenerator) {
        match name_parts {
            &[] => {}

            &[name] => {
                self.insert_asset(asset_id, name);
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
                child.insert(asset_id, &name_parts[1..], id_generator);
                self.mark_used();
            }
        }
    }

    fn update_asset(&mut self, asset: &DataAsset, id_generator: &mut AssetTreeNodeIdGenerator) {
        if ! asset.name.contains(ASSET_NAME_SEPARATOR) {  // avoid splitting the string if we can
            self.insert_asset(asset.id, &asset.name);
        } else {
            let name_parts: Vec<&str> = asset.name.split(ASSET_NAME_SEPARATOR).collect();
            self.insert(asset.id, &name_parts, id_generator);
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

    pub fn get_node_name(&self, node_id: AssetTreeNodeId) -> Option<String> {
        let mut parts = Vec::new();
        if self.get_node_name_parts(node_id, &mut parts) && ! parts.is_empty() {
            Some(parts.into_iter().rev().collect::<Vec<String>>().join(ASSET_NAME_SEPARATOR))
        } else {
            None
        }
    }

    pub fn show_inside(&self, id_prefix: &str, ui: &mut egui::Ui,
                       folder_menu: &mut impl FnMut(&egui::Response, &AssetTreeContainer),
                       show_item: &mut impl FnMut(&mut egui::Ui, &AssetTreeContainer, &AssetTreeItem)) {
        let tree_node_id = ui.make_persistent_id(format!("{}_{}", id_prefix, self.node_id.id));
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, self.level == 0);
        let mut toggle_node_open = false;
        let mut header_resp = node.show_header(ui, |ui| {
            let header = ui.add(egui::Label::new(&self.name).selectable(false).sense(egui::Sense::click()));
            folder_menu(&header, self);
            toggle_node_open = header.clicked();
        });
        if toggle_node_open {
            header_resp.toggle();
        }
        header_resp.body(|ui| {
            for tree in &self.containers {
                tree.show_inside(id_prefix, ui, folder_menu, show_item);
            }
            for asset_node in &self.assets {
                show_item(ui, self, asset_node);
            }
        });
    }
}

pub struct AssetTree {
    roots: HashMap<DataAssetType, AssetTreeContainer>,
    id_generator: AssetTreeNodeIdGenerator,
}

impl AssetTree {
    pub fn new() -> Self {
        let mut id_generator = AssetTreeNodeIdGenerator::new();
        let mut roots = HashMap::new();
        for def in ASSET_DEFS {
            let tree = AssetTreeContainer::new(id_generator.generate_id(), def.tree_root_item.to_owned(), 0);
            roots.insert(def.asset_type, tree);
        }
        AssetTree {
            id_generator,
            roots,
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
