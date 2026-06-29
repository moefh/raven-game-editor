use std::collections::{
    HashMap,
    HashSet
};

use crate::app::{
    WindowContext,
    SimpleAssetTree,
    AssetTreeItem,
    AssetTreeContainer,
};
use crate::data_asset::{
    Room,
    World,
    WorldRegion,
    Tileset,
    MapData,
    DataAssetId,
    AssetList,
};
use super::super::widgets::RoomViewWidget;
use super::super::{
    RoomSize,
    AssetEditorBase,
};

pub struct RoomSelectionDialog {
    pub open: bool,
    pub region_index: usize,
    pub sel_room_ids: HashSet<DataAssetId>,
    pub display_room_id: Option<DataAssetId>,
    pub room_tree: Option<SimpleAssetTree>,
}

impl RoomSelectionDialog {
    pub fn new() -> Self {
        RoomSelectionDialog {
            open: false,
            region_index: 0,
            sel_room_ids: HashSet::new(),
            display_room_id: None,
            room_tree: None,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_world_region_rooms")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, world: &World, region_index: usize, rooms: &AssetList<Room>) {
        if let Some(region) = world.regions.get(region_index) {
            self.sel_room_ids.clear();
            for &room_id in region.rooms.iter() {
                self.sel_room_ids.insert(room_id);
            }
            self.display_room_id = region.rooms.first().copied();
            self.room_tree = Some(SimpleAssetTree::from_assets(format!("room_sel_{}", world.asset.id), "Available Rooms", rooms.iter()));
            self.region_index = region_index;
            self.open = true;
            wc.set_dialog_open(Self::id(), self.open);
        }
    }

    fn fix_room_indices(region: &mut WorldRegion, old_rooms: &[DataAssetId]) {
        let mut conv = HashMap::<Option<u8>, Option<u8>>::new();
        conv.insert(None, None);
        for (old_index, room_id) in old_rooms.iter().enumerate() {
            let old_index = Some((old_index & 0xff) as u8);
            let new_index = region.rooms.iter().position(|new_room_id| new_room_id == room_id).map(|i| (i & 0xff) as u8);
            conv.insert(old_index, new_index);
        }
        for block in region.blocks.iter_mut() {
            if let Some(&index) = conv.get(block) {
                *block = index;
            }
        }
    }

    fn confirm(&mut self, world: &mut World) -> bool {
        if let Some(region) = world.regions.get_mut(self.region_index) {
            let old_rooms = region.rooms.clone();

            // remove rooms not in selection
            let size = region.rooms.len();
            region.rooms.retain(|room_id| self.sel_room_ids.contains(room_id));
            let changed = size != region.rooms.len();

            // add selected rooms that are not in the region
            let size = region.rooms.len();
            for room_id in self.sel_room_ids.iter() {
                if ! region.rooms.contains(room_id) {
                    region.rooms.push(*room_id);
                }
            }
            let changed = changed || (size != region.rooms.len());
            if changed {
                region.rooms.sort();
                Self::fix_room_indices(region, &old_rooms);
            }
            changed
        } else {
            false
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, world: &mut World, rooms: &AssetList<Room>,
                maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) -> bool {
        let mut rooms_changed = false;
        let modal_response = AssetEditorBase::show_dialog_window(wc, Self::id(), 550.0, "World Region Rooms", |ui, wc| {
            let asset_id = world.asset.id;

            egui::Panel::bottom(format!("editor_panel_{}_rooms_bot", asset_id)).show_separator_line(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        rooms_changed = self.confirm(world);
                        ui.close();
                    }
                });
            });

            egui::Panel::left(format!("editor_panel_{}_rooms_left", asset_id))
                .resizable(false)
                .show_separator_line(false)
                .min_size(200.0)
                .show_inside(ui, |ui| {
                    let mut add_room: Option<DataAssetId> = None;
                    let mut remove_room: Option<DataAssetId> = None;
                    let mut display_room: Option<DataAssetId> = None;
                    if let Some(room_tree) = &self.room_tree {
                        let mut show_folder = |ui: &mut egui::Ui, folder: &AssetTreeContainer| -> egui::Response {
                            ui.add(egui::Label::new(&folder.name).selectable(false).sense(egui::Sense::click()))
                        };
                        let mut show_item = |ui: &mut egui::Ui, _folder: &AssetTreeContainer, asset_item: &AssetTreeItem| {
                            ui.horizontal(|ui| {
                                let mut checked = self.sel_room_ids.contains(&asset_item.id);
                                let old_checked = checked;
                                ui.add(egui::Checkbox::without_text(&mut checked));
                                if checked != old_checked {
                                    display_room = Some(asset_item.id);
                                    if checked {
                                        add_room = Some(asset_item.id);
                                    } else  {
                                        remove_room = Some(asset_item.id);
                                    }
                                }
                                if ui.button(&asset_item.name).clicked() {
                                    display_room = Some(asset_item.id);
                                }
                            });
                        };
                        room_tree.show_inside(ui, true, &mut show_folder, &mut show_item);
                    }
                    if let Some(room_id) = add_room { self.sel_room_ids.insert(room_id); }
                    if let Some(room_id) = remove_room { self.sel_room_ids.remove(&room_id); }
                    if display_room.is_some() { self.display_room_id = display_room; }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(room) = self.display_room_id.and_then(|room_id| rooms.get(&room_id)) {
                    let room_size = RoomSize::from_room(room, maps);
                    ui.label(format!("Room: {} ({}x{})", room.asset.name, room_size.width, room_size.height));
                    RoomViewWidget::show(ui, wc, room, maps, tilesets);
                }
            });
            rooms_changed
        });
        if modal_response.should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
            return modal_response.inner;
        }
        false
    }
}
