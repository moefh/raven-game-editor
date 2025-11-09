use crate::IMAGES;
use crate::misc::WindowContext;
use crate::data_asset::{
    Room, RoomMap, RoomEntity, MapData, DataAssetId, GenericAsset,
    SpriteAnimation, AssetIdCollection, AssetIdList, AssetList,
};

pub struct RoomEditor {
    pub asset: super::DataAssetEditor,
    pub selected_map_id: Option<super::DataAssetId>,
    pub selected_animation_id: Option<super::DataAssetId>,
}

fn get_ui_selectable_asset_id(sel_id: Option<DataAssetId>, ids: &AssetIdList) -> Option<DataAssetId> {
    if sel_id.is_some() {
        sel_id
    } else {
        ids.get_first()
    }
}

impl RoomEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        RoomEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
            selected_map_id: None,
            selected_animation_id: None,
        }
    }

    pub fn show(&mut self, wc: &WindowContext, room: &mut Room,
                asset_ids: &AssetIdCollection, maps: &AssetList<MapData>, animations: &AssetList<SpriteAnimation>) {
        let asset_id = room.asset.id;
        let title = format!("{} - Room", room.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Room", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                //...
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", room.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.label(format!("maps referenced: {}", room.maps.len()));
                if let Some(map_id) = get_ui_selectable_asset_id(self.selected_map_id, &asset_ids.maps) {
                    let map_name = if let Some(map) = maps.get(&map_id) {
                        &map.asset.name
                    } else {
                        "??"
                    };
                    ui.horizontal(|ui| {
                        let mut sel_map_id = map_id;
                        egui::ComboBox::from_id_salt(format!("room_editor_map_combo_{}", asset_id))
                            .selected_text(map_name)
                            .show_ui(ui, |ui| {
                                for map_id in asset_ids.maps.iter() {
                                    if let Some(map) = maps.get(map_id) {
                                        ui.selectable_value(&mut sel_map_id, map.asset.id, &map.asset.name);
                                    }
                                }
                            });
                        if ui.button("Add").clicked() && ! room.maps.iter().any(|m| m.map_id == map_id) {
                            room.maps.push(RoomMap {
                                name: String::new(),
                                x: 0,
                                y: 0,
                                map_id,
                            });
                        }
                        self.selected_map_id = Some(sel_map_id);
                    });
                } else {
                    ui.label("no maps to select!");
                }

                ui.label(format!("entities: {}", room.entities.len()));
                if let Some(animation_id) = get_ui_selectable_asset_id(self.selected_animation_id, &asset_ids.animations) {
                    let animation_name = if let Some(animation) = animations.get(&animation_id) {
                        &animation.asset.name
                    } else {
                        "??"
                    };
                    ui.horizontal(|ui| {
                        let mut sel_animation_id = animation_id;
                        egui::ComboBox::from_id_salt(format!("room_editor_animation_combo_{}", asset_id))
                            .selected_text(animation_name)
                            .show_ui(ui, |ui| {
                                for animation_id in asset_ids.animations.iter() {
                                    if let Some(animation) = animations.get(animation_id) {
                                        ui.selectable_value(&mut sel_animation_id, animation.asset.id, &animation.asset.name);
                                    }
                                }
                            });
                        if ui.button("Add").clicked() {
                            room.entities.push(RoomEntity {
                                name: String::new(),
                                x: 0,
                                y: 0,
                                data0: 0,
                                data1: 0,
                                data2: 0,
                                data3: 0,
                                animation_id,
                            });
                        }
                        self.selected_animation_id = Some(sel_animation_id);
                    });
                } else {
                    ui.label("no animations to select!");
                }
            });
        });
    }
}
