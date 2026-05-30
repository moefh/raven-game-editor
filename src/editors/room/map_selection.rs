use std::collections::HashSet;

use crate::app::{WindowContext, AssetTreeContainer, AssetTreeItem};
use crate::data_asset::{
    Room, RoomMap, MapData, Tileset,
    DataAssetId, AssetList,
};
use super::super::widgets::MapViewWidget;

pub struct MapSelectionDialog {
    pub open: bool,
    pub sel_map_ids: HashSet<DataAssetId>,
    pub display_map_id: Option<DataAssetId>,
    pub map_tree: Option<AssetTreeContainer>,
}

impl MapSelectionDialog {
    pub fn new() -> Self {
        MapSelectionDialog {
            open: false,
            sel_map_ids: HashSet::new(),
            display_map_id: None,
            map_tree: None,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_room_maps")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, room: &Room, maps: &AssetList<MapData>) {
        self.sel_map_ids.clear();
        for map in &room.maps {
            self.sel_map_ids.insert(map.map_id);
        }
        self.display_map_id = room.maps.first().map(|m| m.map_id);
        self.map_tree = Some(AssetTreeContainer::from_assets(maps, "Available Maps".to_owned()));
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, room: &mut Room) -> bool {
        // remove maps not in selection
        let size = room.maps.len();
        room.maps.retain(|m| self.sel_map_ids.contains(&m.map_id));
        let changed = size != room.maps.len();

        // add selected maps that are not in the room
        let size = room.maps.len();
        for &map_id in self.sel_map_ids.iter() {
            if ! room.maps.iter().any(|m| m.map_id == map_id) {
                room.maps.push(RoomMap {
                    x: 0,
                    y: 0,
                    map_id,
                });
            }
        }
        let changed = changed || (size != room.maps.len());

        if changed {
            room.maps.sort_by_key(|m| m.map_id);
        }
        changed
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) -> bool {
        let mut maps_changed = false;
        let modal_response = egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);

            let asset_id = room.asset.id;
            egui::Panel::top(format!("editor_panel_{}_maps_top", asset_id)).show_inside(ui, |ui| {
                ui.add_space(2.0);
                ui.heading("Map Selection");
                ui.add_space(2.0);
            });

            egui::Panel::bottom(format!("editor_panel_{}_maps_bot", asset_id)).show_separator_line(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        maps_changed = self.confirm(room);
                        ui.close();
                    }
                });
            });

            egui::Panel::left(format!("editor_panel_{}_maps_left", asset_id))
                .resizable(false)
                .show_separator_line(false)
                .min_size(200.0)
                .show_inside(ui, |ui| {
                    let mut add_map: Option<DataAssetId> = None;
                    let mut remove_map: Option<DataAssetId> = None;
                    let mut display_map: Option<DataAssetId> = None;
                    if let Some(map_tree) = &self.map_tree {
                        let mut folder_menu = |_header: &egui::Response, _folder: &AssetTreeContainer| {};
                        let mut show_item = |ui: &mut egui::Ui, _folder: &AssetTreeContainer, asset_item: &AssetTreeItem| {
                            ui.horizontal(|ui| {
                                let mut checked = self.sel_map_ids.contains(&asset_item.id);
                                let old_checked = checked;
                                ui.add(egui::Checkbox::without_text(&mut checked));
                                if checked != old_checked {
                                    display_map = Some(asset_item.id);
                                    if checked {
                                        add_map = Some(asset_item.id);
                                    } else  {
                                        remove_map = Some(asset_item.id);
                                    }
                                }
                                if ui.button(&asset_item.name).clicked() {
                                    display_map = Some(asset_item.id);
                                }
                            });
                        };
                        map_tree.show_inside(&format!("map_sel_{}", asset_id), ui, true, &mut folder_menu, &mut show_item);
                    }
                    if let Some(map_id) = add_map { self.sel_map_ids.insert(map_id); }
                    if let Some(map_id) = remove_map { self.sel_map_ids.remove(&map_id); }
                    if display_map.is_some() { self.display_map_id = display_map; }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(map_data) = self.display_map_id.and_then(|map_id| maps.get(&map_id)) &&
                    let Some(tileset) = tilesets.get(&map_data.tileset_id) {
                        ui.label(format!("Map: {} ({}x{})", map_data.asset.name, map_data.width, map_data.height));
                        MapViewWidget::show(ui, wc, map_data, tileset);
                    }
            });
            maps_changed
        });
        if modal_response.should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
            return modal_response.inner;
        }
        false
    }
}
