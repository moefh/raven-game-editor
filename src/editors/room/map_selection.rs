use std::collections::HashSet;

use crate::app::WindowContext;
use crate::data_asset::{
    Room, RoomMap, MapData, Tileset,
    DataAssetId, AssetIdList, AssetList,
};
use super::super::widgets::MapViewWidget;

pub struct MapSelectionDialog {
    pub open: bool,
    pub sel_map_ids: HashSet<DataAssetId>,
    pub display_map_id: Option<DataAssetId>,
}

impl MapSelectionDialog {
    pub fn new() -> Self {
        MapSelectionDialog {
            open: false,
            sel_map_ids: HashSet::new(),
            display_map_id: None,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_room_maps")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, room: &Room) {
        self.sel_map_ids.clear();
        for map in &room.maps {
            self.sel_map_ids.insert(map.map_id);
        }
        self.display_map_id = room.maps.first().map(|m| m.map_id);
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
            room.maps.sort_by(|m1, m2| m1.map_id.cmp(&m2.map_id));
        }
        changed
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, all_map_ids: &AssetIdList,
                maps: &AssetList<MapData>, tilesets: &AssetList<Tileset>) -> bool {
        let mut maps_changed = false;
        let modal_response = egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);

            let asset_id = room.asset.id;
            egui::TopBottomPanel::top(format!("editor_panel_{}_maps_top", asset_id)).show_inside(ui, |ui| {
                ui.add_space(2.0);
                ui.heading("Map Selection");
                ui.add_space(2.0);
            });

            egui::TopBottomPanel::bottom(format!("editor_panel_{}_maps_bot", asset_id)).show_separator_line(false).show_inside(ui, |ui| {
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

            egui::SidePanel::left(format!("editor_panel_{}_maps_left", asset_id))
                .resizable(false)
                .show_separator_line(false)
                .min_width(200.0)
                .show_inside(ui, |ui| {
                    ui.add_space(5.0);
                    ui.label("Available maps:");
                    ui.add_space(5.0);
                    egui::ScrollArea::both().auto_shrink([false, true]).show(ui, |ui| {
                        for &map_id in all_map_ids.iter() {
                            if let Some(map) = maps.get(&map_id) {
                                ui.horizontal(|ui| {
                                    let mut checked = self.sel_map_ids.contains(&map_id);
                                    let old_checked = checked;
                                    ui.add(egui::Checkbox::without_text(&mut checked));
                                    if checked != old_checked {
                                        self.display_map_id = Some(map_id);
                                        if checked {
                                            self.sel_map_ids.insert(map_id);
                                        } else  {
                                            self.sel_map_ids.remove(&map_id);
                                        }
                                    }
                                    if ui.button(&map.asset.name).clicked() {
                                        self.display_map_id = Some(map_id);
                                    }
                                });
                            }
                        }
                    });
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
