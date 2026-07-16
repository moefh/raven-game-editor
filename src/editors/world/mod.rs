mod properties;
mod room_selection;
mod region_properties;

use crate::misc::IMAGES;
use crate::app::{
    menu_item,
    menu_item_no_image,
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
use super::{
    world_grid,
    AssetEditorBase,
};
use super::widgets::{
    WorldEditorWidget,
    WorldRegionEditorWidget,
    RoomGridViewWidget,
};

#[derive(Copy, Clone)]
enum ShiftDirection {
    Left,
    Right,
    Up,
    Down,
}

pub struct WorldEditorAssetLists<'a> {
    pub rooms: &'a mut AssetList<Room>,
    pub maps: &'a AssetList<MapData>,
    pub tilesets: &'a AssetList<Tileset>,
}

impl<'a> WorldEditorAssetLists<'a> {
    pub fn new(rooms: &'a mut AssetList<Room>, maps: &'a AssetList<MapData>, tilesets: &'a AssetList<Tileset>) -> Self {
        WorldEditorAssetLists {
            rooms,
            maps,
            tilesets,
        }
    }
}

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
        egui::Panel::bottom(format!("editor_panel_{}_bottom", world.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let world_size = world_grid::get_world_size(world);

            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            ui.label(format!(
                "{} bytes [{}x{} blocks, {} region{}] {}",
                world.data_size(),
                world_size.0,
                world_size.1,
                world.regions.len(),
                if world.regions.len() != 1 { "s" } else { "" },
                dirty,
            ));
        });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        world: &mut World,
        assets: &mut WorldEditorAssetLists,
    ) {
        self.dialogs.show(wc, &mut self.editor, world, assets);

        self.base.show_window(wc, world, [600.0, 350.0], [700.0, 450.0], |ui, wc, world, base| {
            Self::show_footer(ui, wc, world, base);
            self.editor.show(ui, wc, &mut self.dialogs, world, assets);
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

    fn show(
        &mut self,
        wc: &mut WindowContext,
        editor: &mut Editor,
        world: &mut World,
        assets: &WorldEditorAssetLists,
    ) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, world);
        }
        if self.region_properties_dialog.open {
            self.region_properties_dialog.show(wc, world);
        }
        if self.room_selection_dialog.open &&
            self.room_selection_dialog.show(wc, world, assets) &&
            let Some(region) = world.regions.get(self.room_selection_dialog.region_index) {
                editor.region_editor.ensure_room_selection_is_valid(region);
            }
    }
}

struct Editor {
    asset_id: DataAssetId,
    selected_tab: EditorTab,
    world_grid: world_grid::WorldGridStore,
    world_editor: WorldEditorWidget,
    region_editor: WorldRegionEditorWidget,
    region_rooms_tree: Option<SimpleAssetTree>,
    highlight_door_info: String,
    done_init: bool,
}

impl Editor {
    const REGION_TREE_PANEL_WIDTH: f32 = 160.0;
    const ROOM_TREE_PANEL_WIDTH: f32 = 200.0;

    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            selected_tab: EditorTab::World,
            world_grid: world_grid::WorldGridStore::new(),
            world_editor: WorldEditorWidget::new(),
            region_editor: WorldRegionEditorWidget::new(),
            region_rooms_tree: None,
            highlight_door_info: String::new(),
            done_init: false,
        }
    }

    fn add_region(&mut self, world: &mut World) {
        let region_index  = world.regions.len();
        world.regions.push(WorldRegion::new("new_region", 0, 0, 16, 8));
        self.select_region(world, region_index);
    }

    fn remove_region(&mut self, world: &mut World, region_index: usize) {
        world.regions.remove(region_index);
        self.world_editor.ensure_room_selection_is_valid(world);
    }

    fn select_region(&mut self, world: &World, region_index: usize) {
        self.world_editor.set_selected_region(Some(region_index));
        if let Some(region) = world.regions.get(region_index) && ! region.rooms.is_empty() {
            self.region_editor.set_selected_room(Some(0));
        } else {
            self.region_editor.set_selected_room(None);
        }
    }

    fn show_menubar(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        world: &mut World,
        rooms: &AssetList<Room>
    ) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("World", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, world);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let has_region = self.world_editor.get_selected_region().is_some();
                    if ui.add_enabled(has_region, menu_item(IMAGES.properties, " Region properties...")).clicked() &&
                        let Some(region_index) = self.world_editor.get_selected_region() {
                            dialogs.region_properties_dialog.set_open(wc, world, region_index);
                        }
                    if ui.add_enabled(has_region, menu_item(IMAGES.room, " Select region rooms...")).clicked() &&
                        let Some(region_index) = self.world_editor.get_selected_region() {
                            dialogs.room_selection_dialog.set_open(wc, world, region_index, rooms);
                        }
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
                if ui.add(menu_item_no_image(" Add region")).clicked() {
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
                        if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                            action = RegionTreeAction::EditRegion(region_index);
                        }
                        if ui.add(menu_item(IMAGES.room, " Select Rooms...")).clicked() {
                            action = RegionTreeAction::SelectRegionRooms(region_index);
                        }
                        ui.separator();
                        if ui.add(menu_item(IMAGES.trash, " Remove region")).clicked() {
                            action = RegionTreeAction::RemoveRegion(region_index);
                        }
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

    fn show_world_tab(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        world: &mut World,
        rooms: &mut AssetList<Room>
    ) {
        egui::Panel::top(format!("editor_panel_{}_world_header", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                if ui.add(egui::Button::image_and_text(IMAGES.lock, "Regions")
                          .selected(self.world_editor.lock_regions)
                          .frame_when_inactive(self.world_editor.lock_regions))
                    .on_hover_text("Lock regions in place").clicked() {
                        self.world_editor.lock_regions = ! self.world_editor.lock_regions;
                    }
                if ui.add(egui::Button::image_and_text(IMAGES.lock, "Doors")
                          .selected(self.world_editor.lock_door_connections)
                          .frame_when_inactive(self.world_editor.lock_door_connections))
                    .on_hover_text("Lock door connections").clicked() {
                        self.world_editor.lock_door_connections = ! self.world_editor.lock_door_connections;
                    }
            });
            ui.add_space(0.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.world_editor.show(ui, wc, world, rooms, &self.world_grid);
        });
    }

    fn show_region_rooms_tree(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        world: &mut World,
        region_index: usize,
        rooms: &AssetList<Room>
    ) -> [RoomTreeAction; 2] {
        let (mut header_action, mut item_action) = (RoomTreeAction::None, RoomTreeAction::None);

        let region = if let Some(region) = world.regions.get_mut(region_index) {
            region
        } else {
            return [header_action, item_action];
        };
        let tree = self.region_rooms_tree.get_or_insert_with(|| {
            SimpleAssetTree::new(format!("editor_{}_region_rooms_tree", self.asset_id), "Rooms")
        });
        let region_rooms = rooms.iter().filter(|room| region.rooms.contains(&room.asset.id));
        tree.update_assets(region_rooms);

        let mut show_folder = |ui: &mut egui::Ui, folder: &AssetTreeContainer| -> egui::Response {
            let resp = ui.add(egui::Button::new(&folder.name).frame_when_inactive(false));
            egui::Popup::context_menu(&resp).show(|ui| {
                if ui.add(menu_item(IMAGES.room, " Select rooms...")).clicked() {
                    header_action = RoomTreeAction::SelectRegionRooms;
                }
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
        tree.show(ui, true, &mut show_folder, &mut show_item);
        [header_action, item_action]
    }

    fn shift_region(&mut self, region: &mut WorldRegion, direction: ShiftDirection) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        const MAX_WIDTH: usize = WorldRegion::MAX_WIDTH as usize;
        const MAX_HEIGHT: usize = WorldRegion::MAX_HEIGHT as usize;
        let region_width = region.width as usize;
        let region_height = region.height as usize;

        // shift
        match direction {
            ShiftDirection::Up => {
                let mut row = [None; MAX_WIDTH];
                row[..].copy_from_slice(&region.blocks[0 .. MAX_WIDTH]);
                region.blocks[..].copy_within(MAX_WIDTH.., 0);
                region.blocks[(region_height-1)*MAX_WIDTH .. region_height*MAX_WIDTH].copy_from_slice(&row[..]);
            }
            ShiftDirection::Down => {
                let mut row = [None; MAX_WIDTH];
                row[..].copy_from_slice(&region.blocks[(region_height-1)*MAX_WIDTH .. region_height*MAX_WIDTH]);
                region.blocks[..].copy_within(0 .. (MAX_WIDTH-1)*MAX_WIDTH, MAX_WIDTH);
                region.blocks[0 .. MAX_WIDTH].copy_from_slice(&row[..]);
            }
            ShiftDirection::Left => {
                let mut row = [None; MAX_HEIGHT];
                for (y, block) in row.iter_mut().enumerate() {
                    *block = region.blocks[y*MAX_WIDTH];
                }
                region.blocks[..].copy_within(1.., 0);
                for (y, block) in row.iter().enumerate() {
                    region.blocks[y*MAX_WIDTH + region_width - 1] = *block;
                }
            }
            ShiftDirection::Right => {
                let mut row = [None; MAX_HEIGHT];
                for (y, block) in row.iter_mut().enumerate() {
                    *block = region.blocks[y*MAX_WIDTH + region_width - 1];
                }
                region.blocks[..].copy_within(0..MAX_WIDTH*MAX_HEIGHT-1, 1);
                for (y, block) in row.iter().enumerate() {
                    region.blocks[y*MAX_WIDTH] = *block;
                }
            }
        }

        // clear unused region
        if region_width < MAX_WIDTH {
            for y in 0..MAX_HEIGHT {
                region.blocks[y*MAX_WIDTH + region_width] = None;
                region.blocks[y*MAX_WIDTH + MAX_WIDTH-1] = None;
            }
        }
        if region_height < MAX_HEIGHT {
            region.blocks[region_height*MAX_WIDTH .. (region_height+1)*MAX_WIDTH].fill(None);
            region.blocks[(MAX_HEIGHT-1)*MAX_WIDTH .. MAX_HEIGHT*MAX_WIDTH].fill(None);
        }
    }

    fn show_region_tab(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        world: &mut World,
        assets: &WorldEditorAssetLists,
    ) {
        let region_index = match self.world_editor.get_selected_region() {
            Some(region_index) => { region_index }
            None => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.label("No region selected");
                });
                return;
            }
        };

        let actions = {
            egui::Panel::top(format!("editor_panel_{}_region_toolbar", self.asset_id)).show(ui, |ui| {
                ui.add_space(2.0);

                if let Some(region) = world.regions.get_mut(region_index) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.arrow_up)).on_hover_text("Shift Up").clicked() {
                            self.shift_region(region, ShiftDirection::Up);
                        }
                        if ui.add(egui::Button::image(IMAGES.arrow_down)).on_hover_text("Shift Down").clicked() {
                            self.shift_region(region, ShiftDirection::Down);
                        }
                        if ui.add(egui::Button::image(IMAGES.arrow_left)).on_hover_text("Shift Left").clicked() {
                            self.shift_region(region, ShiftDirection::Left);
                        }
                        if ui.add(egui::Button::image(IMAGES.arrow_right)).on_hover_text("Shift Right").clicked() {
                            self.shift_region(region, ShiftDirection::Right);
                        }
                    });
                }

                ui.add_space(0.0);
            });

            let actions = egui::Panel::right(format!("editor_panel_{}_region_rooms_tree", self.asset_id))
                .resizable(false)
                .show(ui, |ui| {
                    let want_height = 70.0_f32.max(ui.available_height() / 2.0);
                    let actions = ui.allocate_ui(egui::Vec2::new(Self::ROOM_TREE_PANEL_WIDTH, want_height), |ui| {
                        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                            self.show_region_rooms_tree(ui, wc, dialogs, world, region_index, assets.rooms)
                        }).inner
                    }).inner;
                    ui.separator();
                    if let Some(region) = world.regions.get(region_index) &&
                        let Some(room_index) = self.region_editor.get_selected_room() &&
                        let Some(room_id) = region.rooms.get(room_index as usize) &&
                        let Some(room) = assets.rooms.get(room_id) {
                            RoomGridViewWidget::show(ui, wc, room, world, assets.maps, &self.world_grid, region_index);
                        } else {
                            ui.label("No room selected");
                        }
                    actions
                }).inner;

            egui::CentralPanel::default().show(ui, |ui| {
                self.region_editor.show(ui, wc, world, &self.world_grid);
            });

            actions
        };
        for action in actions {
            match action {
                RoomTreeAction::None => {}
                RoomTreeAction::SelectRoom(index) => { self.region_editor.set_selected_room(Some((index & 0xff) as u8)); }
                RoomTreeAction::SelectRegionRooms => {
                    if let Some(region_index) = self.world_editor.get_selected_region() {
                        dialogs.room_selection_dialog.set_open(wc, world, region_index, assets.rooms);
                    }
                }
            }
        }
    }

    pub fn init(&mut self, world: &World) {
        if ! world.regions.is_empty() {
            self.world_editor.set_selected_region(Some(0));
        }
    }

    fn update_highlight_door_info(&mut self, world: &World, rooms: &AssetList<Room>) {
        self.highlight_door_info.clear();
        let (src_door, dest_door, dragging) = match self.selected_tab {
            EditorTab::World => {
                if let Some(door_index) = self.world_editor.get_dragged_door_connection_src_index() {
                    let src_door = self.world_grid.doors.get(door_index);
                    let dest_door = self.world_editor.highlight_door_index.and_then(|door_index| self.world_grid.doors.get(door_index));
                    (src_door, dest_door, true)
                } else if let Some(door) = self.world_editor.highlight_door_index.and_then(|door_index| self.world_grid.doors.get(door_index)) {
                    (Some(door), door.get_dest_door(&self.world_grid.doors), false)
                } else {
                    (None, None, false)
                }
            }
            EditorTab::Region => {
                if let Some(door) = self.region_editor.highlight_door_index.and_then(|door_index| self.world_grid.doors.get(door_index)) {
                    (Some(door), door.get_dest_door(&self.world_grid.doors), false)
                } else {
                    (None, None, false)
                }
            }
        };
        if let Some(src_door) = src_door {
            if dragging {
                self.highlight_door_info.push_str("Connecting: ");
            } else {
                self.highlight_door_info.push_str("Door: ");
            }
            src_door.get_info_with_region(&mut self.highlight_door_info, world, rooms);
            self.highlight_door_info.push_str(" \u{25b6} ");
            if let Some(dest_door) = dest_door {
                if dest_door.index == src_door.index {
                    self.highlight_door_info.push_str(if dragging { "..." } else { "[SELF]" });
                } else if dest_door.is_in_region(src_door.region_index) {
                    dest_door.get_info(&mut self.highlight_door_info, rooms);
                } else {
                    dest_door.get_info_with_region(&mut self.highlight_door_info, world, rooms);
                }
            } else {
                self.highlight_door_info.push_str(if dragging { "..." } else { "[INVALID DESTINATION]" });
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        world: &mut World,
        assets: &mut WorldEditorAssetLists,
    ) {
        self.world_grid.update(world, assets.rooms, assets.maps);
        if ! self.done_init {
            self.done_init = true;
            self.init(world);
        }
        self.region_editor.set_selected_region(self.world_editor.get_selected_region());

        self.show_menubar(ui, wc, dialogs, world, assets.rooms);

        // region tree
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id))
            .resizable(false)
            .show(ui, |ui| {
                ui.add_space(5.0);
                ui.allocate_ui(egui::Vec2::new(Self::REGION_TREE_PANEL_WIDTH, ui.available_height()), |ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        match self.show_region_tree(ui, world) {
                            RegionTreeAction::None => {}
                            RegionTreeAction::AddRegion => { self.add_region(world); }
                            RegionTreeAction::SelectRegion(index) => { self.select_region(world, index); }
                            RegionTreeAction::RemoveRegion(index) => { self.remove_region(world, index); }
                            RegionTreeAction::EditRegion(index) => { dialogs.region_properties_dialog.set_open(wc, world, index); }
                            RegionTreeAction::SelectRegionRooms(index) => {
                                dialogs.room_selection_dialog.set_open(wc, world, index, assets.rooms);
                            }
                        }
                    });
                });
            });

        // bottom bar
        egui::Panel::bottom(format!("editor_panel_{}_window_bottom", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if let Some(region_index) = self.world_editor.get_selected_region() && let Some(region) = world.regions.get_mut(region_index) {
                    ui.label(format!("{}x{} blocks", region.width, region.height));
                } else {
                    ui.label("No region selected");
                }
                ui.separator();
                self.update_highlight_door_info(world, assets.rooms);
                ui.label(&self.highlight_door_info);
            });
        });

        // tabs
        egui::Panel::top(format!("editor_panel_{}_tabs", self.asset_id)).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::World), "World").clicked() {
                    self.selected_tab = EditorTab::World;
                }
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::Region), "Region").clicked() {
                    self.selected_tab = EditorTab::Region;
                }
            });
        });
        match self.selected_tab {
            EditorTab::World => { self.show_world_tab(ui, wc, dialogs, world, assets.rooms); }
            EditorTab::Region => { self.show_region_tab(ui, wc, dialogs, world, assets); }
        }
    }
}
