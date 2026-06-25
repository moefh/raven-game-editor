mod properties;
mod room_selection;
mod region_properties;

use crate::misc::IMAGES;
use crate::app::{
    WindowContext,
    SimpleAssetTree,
    AssetTreeContainer,
    AssetTreeItem,
};
use crate::data_asset::{
    Room,
    World,
    WorldRegion,
    Tileset,
    MapData,
    AssetList,
    DataAssetId,
    GenericAsset,
};

use properties::PropertiesDialog;
use room_selection::RoomSelectionDialog;
use region_properties::RegionPropertiesDialog;
use super::AssetEditorBase;
use super::widgets::{
    WorldEditorWidget,
    WorldRegionEditorWidget,
};

enum EditorTab {
    World,
    Region,
}

enum RegionTreeAction {
    None,
    AddRegion,
    SelectRegion(usize),
    RemoveRegion(usize),
    EditRegion(usize),
    SelectRegionRooms(usize),
}

enum RoomTreeAction {
    None,
    SelectRoom(usize),
    SelectRegionRooms,
}

pub struct WorldEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl WorldEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        WorldEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _world: &mut World) {
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, world: &World, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, world.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", world.asset.id)).frame(bottom_frame).show_inside(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            ui.label(format!(
                "{} bytes [{} region{}] {}",
                world.data_size(),
                world.regions.len(),
                if world.regions.len() != 1 { "s" } else { "" },
                dirty,
            ));
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, world: &mut World, rooms: &AssetList<Room>,
                maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) {
        self.dialogs.show(wc, &mut self.editor, world, rooms, maps, tilesets);

        self.base.show_window(wc, world, [500.0, 250.0], [650.0, 300.0], |ui, wc, world, base| {
            Self::show_footer(ui, wc, world, base);
            self.editor.show(ui, wc, &mut self.dialogs, world, rooms);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    room_selection_dialog: RoomSelectionDialog,
    region_properties_dialog: RegionPropertiesDialog,
}

impl Dialogs {
    pub fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            room_selection_dialog: RoomSelectionDialog::new(),
            region_properties_dialog: RegionPropertiesDialog::new(),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, _editor: &mut Editor, world: &mut World,
            rooms: &AssetList<Room>, maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, world);
        }
        if self.region_properties_dialog.open {
            self.region_properties_dialog.show(wc, world);
        }
        if self.room_selection_dialog.open {
            self.room_selection_dialog.show(wc, world, rooms, maps, tilesets);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    selected_tab: EditorTab,
    world_editor: WorldEditorWidget,
    region_editor: WorldRegionEditorWidget,
    region_rooms_tree: Option<SimpleAssetTree>,
}

impl Editor {
    const REGION_TREE_PANEL_WIDTH: f32 = 200.0;
    const ROOM_TREE_PANEL_WIDTH: f32 = 200.0;

    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            selected_tab: EditorTab::World,
            world_editor: WorldEditorWidget::new(),
            region_editor: WorldRegionEditorWidget::new(),
            region_rooms_tree: None,
        }
    }

    fn add_region(&self, world: &mut World) {
        world.regions.push(WorldRegion::new("new_region", 0, 0, 16, 8));
    }

    fn remove_region(&self, world: &mut World, region_index: usize) {
        world.regions.remove(region_index);
    }

    fn select_region(&mut self, world: &World, region_index: usize) {
        self.world_editor.set_selected_region(Some(region_index));
        if let Some(region) = world.regions.get(region_index) && ! region.rooms.is_empty() {
            self.region_editor.set_selected_room(Some(0));
        } else {
            self.region_editor.set_selected_room(None);
        }
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext,
                    dialogs: &mut Dialogs, world: &mut World, rooms: &AssetList<Room>) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("World", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, world);
                        }
                    });
                });

                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        if self.world_editor.get_selected_region().is_none() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.room).max_width(14.0).max_height(14.0));
                        if ui.button("Select region rooms...").clicked() &&
                            let Some(region_index) = self.world_editor.get_selected_region() {
                                dialogs.room_selection_dialog.set_open(wc, world, region_index, rooms);
                            }
                    });
                });
            });
        });
    }

    fn show_region_tree(&mut self, ui: &mut egui::Ui, world: &World) -> RegionTreeAction {
        let mut action = RegionTreeAction::None;
        let tree_node_id = ui.make_persistent_id(format!("editor_{}_region_tree", self.asset_id));
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tree_node_id, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Regions").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                if ui.button("Add region").clicked() {
                    action = RegionTreeAction::AddRegion;
                }
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (region_index, region) in world.regions.iter().enumerate() {
                let item = ui.horizontal(|ui| {
                    let mut selected = Some(region_index) == self.world_editor.get_selected_region();
                    let resp = ui.toggle_value(&mut selected, &region.name);
                    if resp.clicked() || resp.secondary_clicked() {
                        action = RegionTreeAction::SelectRegion(region_index);
                    }
                    egui::Popup::context_menu(&resp).show(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                            if ui.button("Properties...").clicked() {
                                action = RegionTreeAction::EditRegion(region_index);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.room).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                            if ui.button("Select Rooms...").clicked() {
                                action = RegionTreeAction::SelectRegionRooms(region_index);
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.trash).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_SIZE)));
                            if ui.button("Remove region").clicked() {
                                action = RegionTreeAction::RemoveRegion(region_index);
                            }
                        });
                    });
                    selected
                });
                if item.inner && self.world_editor.has_selected_region_changed() {
                    item.response.scroll_to_me(None);
                    self.world_editor.clear_selected_region_changed();
                }
            }
        });
        action
    }

    fn show_world_editor(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, _dialogs: &mut Dialogs, world: &mut World) {
        self.world_editor.show(ui, wc, world);
    }

    fn show_region_rooms_tree(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext,
                              _dialogs: &mut Dialogs, region: &mut WorldRegion, rooms: &AssetList<Room>) -> [RoomTreeAction; 2] {
        let tree = self.region_rooms_tree.get_or_insert_with(|| {
            SimpleAssetTree::new(format!("editor_{}_region_rooms_tree", self.asset_id), "Rooms")
        });
        let region_rooms = rooms.iter().filter(|room| region.rooms.contains(&room.asset.id));
        tree.update_assets(region_rooms);

        let (mut header_action, mut item_action) = (RoomTreeAction::None, RoomTreeAction::None);
        let mut show_folder = |ui: &mut egui::Ui, folder: &AssetTreeContainer| -> egui::Response {
            let resp = ui.add(egui::Button::new(&folder.name).frame_when_inactive(false));
            egui::Popup::context_menu(&resp).show(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(IMAGES.room).max_size(egui::Vec2::splat(crate::app::IMAGE_TREE_CTX_MENU_SIZE)));
                    if ui.button("Select rooms...").clicked() {
                        header_action = RoomTreeAction::SelectRegionRooms;
                    }
                });
            });
            resp
        };
        let mut show_item = |ui: &mut egui::Ui, _folder: &AssetTreeContainer, asset_item: &AssetTreeItem| {
            if let Some(room_index) = region.rooms.iter().position(|&room_id| room_id == asset_item.id) {
                let mut selected = Some(room_index) == self.region_editor.get_selected_room().map(|i| i as usize);
                let resp = ui.toggle_value(&mut selected, &asset_item.name);
                if resp.clicked() || resp.secondary_clicked() {
                    item_action = RoomTreeAction::SelectRoom(room_index);
                }
            } else {
                ui.label(&asset_item.name);
            }
        };
        tree.show_inside(ui, true, &mut show_folder, &mut show_item);
        [header_action, item_action]
    }

    fn show_region_editor(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                          world: &mut World, rooms: &AssetList<Room>) {
        let actions = {
            let region = match self.world_editor.get_selected_region().and_then(|region_index| world.regions.get_mut(region_index)) {
                Some(r) => { r }
                None => { return; }
            };
            egui::Panel::top(format!("editor_panel_{}_region_header", self.asset_id)).show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut region.name);
                });
            });

            let action = egui::Panel::right(format!("editor_panel_{}_region_rooms_tree", self.asset_id))
                .min_size(Self::ROOM_TREE_PANEL_WIDTH)
                .show_inside(ui, |ui| {
                    self.show_region_rooms_tree(ui, wc, dialogs, region, rooms)
                }).inner;

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.region_editor.show(ui, wc, region);
            });

            action
        };
        for action in actions {
            match action {
                RoomTreeAction::None => {}
                RoomTreeAction::SelectRoom(index) => { self.region_editor.set_selected_room(Some((index & 0xff) as u8)); }
                RoomTreeAction::SelectRegionRooms => {
                    if let Some(region_index) = self.world_editor.get_selected_region() {
                        dialogs.room_selection_dialog.set_open(wc, world, region_index, rooms);
                    }
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, world: &mut World, rooms: &AssetList<Room>) {
        self.show_menubar(ui, wc, dialogs, world, rooms);

        // region tree
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id))
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.allocate_ui(egui::Vec2::new(Self::REGION_TREE_PANEL_WIDTH, ui.available_height()), |ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        match self.show_region_tree(ui, world) {
                            RegionTreeAction::None => {}
                            RegionTreeAction::AddRegion => { self.add_region(world); }
                            RegionTreeAction::SelectRegion(index) => { self.select_region(world, index); }
                            RegionTreeAction::RemoveRegion(index) => { self.remove_region(world, index); }
                            RegionTreeAction::EditRegion(index) => { dialogs.region_properties_dialog.set_open(wc, world, index); }
                            RegionTreeAction::SelectRegionRooms(index) => { dialogs.room_selection_dialog.set_open(wc, world, index, rooms); }
                        }
                    });
                });
            });

        // tabs
        egui::Panel::top(format!("editor_panel_{}_tabs", self.asset_id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::World), "World").clicked() {
                    self.selected_tab = EditorTab::World;
                }
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::Region), "Region").clicked() {
                    self.selected_tab = EditorTab::Region;
                }
            });
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            match self.selected_tab {
                EditorTab::World => { self.show_world_editor(ui, wc, dialogs, world); }
                EditorTab::Region => { self.show_region_editor(ui, wc, dialogs, world, rooms); }
            }
        });
    }
}
