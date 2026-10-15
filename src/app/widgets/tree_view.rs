pub trait TreeViewItemTrait {
    fn name(&self) -> &str;
}

pub struct TreeViewItem {
    pub item_id: usize,
    pub display_name: String,
    full_name: String,
    used: bool,
}

impl TreeViewItem {
    fn new(item_id: usize, full_name: String, display_name: String) -> Self {
        TreeViewItem {
            item_id,
            full_name,
            display_name,
            used: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TreeViewFolderId {
    id: u32,
}

struct TreeViewFolderIdGenerator {
    next: u32,
}

impl TreeViewFolderIdGenerator {
    pub fn new() -> Self {
        TreeViewFolderIdGenerator {
            next: 0,
        }
    }

    fn generate_id(&mut self) -> TreeViewFolderId {
        let id = self.next;
        self.next += 1;
        TreeViewFolderId {
            id,
        }
    }
}

pub struct TreeViewFolder {
    egui_id: egui::Id,
    pub node_id: TreeViewFolderId,
    pub display_name: String,
    pub level: usize,
    folders: Vec<TreeViewFolder>,
    items: Vec<TreeViewItem>,
    used: bool,
}

impl TreeViewFolder {
    fn new(egui_id: egui::Id, node_id: TreeViewFolderId, display_name: String, level: usize) -> Self {
        TreeViewFolder {
            egui_id,
            node_id,
            display_name,
            level,
            used: true,
            folders: Vec::new(),
            items: Vec::new(),
        }
    }

    fn sort(&mut self) {
        self.items.sort_by(|a, b| { a.display_name.cmp(&b.display_name) });
        self.folders.sort_by(|a, b| { a.display_name.cmp(&b.display_name) });
        for folder in &mut self.folders {
            folder.sort();
        }
    }

    fn mark_tree_used(&mut self, used: bool) {
        self.used = used;
        for folder in &mut self.folders {
            folder.mark_tree_used(used);
        }
        for item in &mut self.items {
            item.used = used;
        }
    }

    fn remove_unused(&mut self) {
        self.items.retain(|c| c.used);
        self.folders.retain(|c| c.used || c.level == 0);
        for folder in &mut self.folders {
            folder.remove_unused();
        }
    }

    fn insert_item(&mut self, item_id: usize, full_name: &str, display_name: &str) {
        match self.items.iter_mut().find(|item| item.item_id == item_id && item.full_name == full_name) {
            Some(a) => {
                a.used = true;
            }
            None => {
                self.items.push(TreeViewItem::new(item_id, full_name.to_owned(), display_name.to_owned()));
            }
        }
        self.used = true;
    }

    fn insert(&mut self, item_id: usize, full_name: &str, name_parts: &[&str], id_generator: &mut TreeViewFolderIdGenerator) {
        match name_parts {
            &[] => {}

            &[name] => {
                self.insert_item(item_id, full_name, name);
            }

            name_parts => {
                let folder = match self.folders.iter_mut().find(|c| c.display_name == name_parts[0]) {
                    Some(folder) => {
                        folder
                    }
                    None => {
                        let node_id = id_generator.generate_id();
                        let folder = TreeViewFolder::new(
                            self.egui_id.with(node_id),
                            node_id,
                            name_parts[0].to_owned(),
                            self.level + 1
                        );
                        self.folders.push_mut(folder)
                    }
                };
                folder.insert(item_id, full_name, &name_parts[1..], id_generator);
                self.used = true;
            }
        }
    }

    fn update_tree(&mut self, items: &[impl TreeViewItemTrait], id_generator: &mut TreeViewFolderIdGenerator) {
        for (item_id, item) in items.iter().enumerate() {
            let item_name = item.name();
            if ! item_name.contains(TreeView::PATH_SEPARATOR) {  // avoid splitting the string if we can
                self.insert_item(item_id, item_name, item_name);
            } else {
                let name_parts: Vec<&str> = item_name.split(TreeView::PATH_SEPARATOR).collect();
                self.insert(item_id, item_name, &name_parts, id_generator);
            }
        }
    }

    fn get_node_name_parts(&self, node_id: TreeViewFolderId, parts: &mut Vec<String>) -> bool {
        let found = if node_id == self.node_id {
            true
        } else {
            self.folders.iter().position(|c| c.get_node_name_parts(node_id, parts)).is_some()
        };
        if found && self.level != 0 {
            parts.push(self.display_name.clone());
        }
        found
    }

    fn get_node_name(&self, node_id: TreeViewFolderId) -> Option<String> {
        let mut parts = Vec::new();
        if self.get_node_name_parts(node_id, &mut parts) && ! parts.is_empty() {
            Some(parts.into_iter().rev().collect::<Vec<String>>().join(TreeView::PATH_SEPARATOR))
        } else {
            None
        }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        is_folder_open: &impl Fn(&TreeViewFolder) -> bool,
        show_folder: &mut impl FnMut(&mut egui::Ui, &TreeViewFolder) -> egui::Response,
        show_item: &mut impl FnMut(&mut egui::Ui, &TreeViewFolder, &TreeViewItem)
    ) {
        let open = is_folder_open(self);
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), self.egui_id, open);
        let mut toggle_node_open = false;
        if self.level == 0 {
            ui.add_space(4.0);
        }
        let mut header_resp = node.show_header(ui, |ui| {
            toggle_node_open = show_folder(ui, self).clicked();
        });
        if toggle_node_open {
            header_resp.toggle();
        }
        header_resp.body(|ui| {
            for folder in &self.folders {
                folder.show(ui, is_folder_open, show_folder, show_item);
            }
            for item in &self.items {
                show_item(ui, self, item);
            }
        });
    }
}

pub struct TreeView {
    root: TreeViewFolder,
    id_generator: TreeViewFolderIdGenerator,
}

impl TreeView {
    pub const PATH_SEPARATOR: &str = "/";

    pub fn new(egui_id_salt: impl AsRef<str>, root_display_name: impl Into<String>) -> Self {
        let mut id_generator = TreeViewFolderIdGenerator::new();
        let root = TreeViewFolder::new(
            egui::Id::new(egui_id_salt.as_ref()),
            id_generator.generate_id(),
            root_display_name.into(),
            0
        );
        TreeView {
            root,
            id_generator,
        }
    }

    pub fn update_items(&mut self, items: &[impl TreeViewItemTrait]) {
        self.root.mark_tree_used(false);
        self.root.update_tree(items, &mut self.id_generator);
        self.root.remove_unused();
        self.root.sort();
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        is_folder_open: &impl Fn(&TreeViewFolder) -> bool,
        show_folder: &mut impl FnMut(&mut egui::Ui, &TreeViewFolder) -> egui::Response,
        show_item: &mut impl FnMut(&mut egui::Ui, &TreeViewFolder, &TreeViewItem)
    ) {
        self.root.show(ui, is_folder_open, show_folder, show_item);
    }

    pub fn get_node_name(&self, node_id: TreeViewFolderId) -> Option<String> {
        self.root.get_node_name(node_id)
    }
}
