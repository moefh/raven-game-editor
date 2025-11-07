use crate::app::WindowContext;
use crate::IMAGES;
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId, GenericAsset};

pub struct MapDataEditor {
    pub asset: super::DataAssetEditor,
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            }
        }
    }

    pub fn show(&mut self, wc: &WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        let title = format!("{} - Map", map_data.asset.name);
        let window = super::create_editor_window(map_data.asset.id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", map_data.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Map", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", map_data.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", map_data.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let cur_tileset_name = if let Some(cur_tileset) = tilesets.get(&map_data.tileset_id) {
                    &cur_tileset.asset.name
                } else {
                    "??"
                };
                egui::ComboBox::from_id_salt(format!("map_editor_tileset_combo_{}", map_data.asset.id))
                    .selected_text(cur_tileset_name)
                    .show_ui(ui, |ui| {
                        for tileset_id in tileset_ids.iter() {
                            if let Some(tileset) = tilesets.get(tileset_id) {
                                ui.selectable_value(&mut map_data.tileset_id, tileset.asset.id, &tileset.asset.name);
                            }
                        }
                    });
                ui.add(egui::Image::new(IMAGES.map_data).max_width(32.0));
            });
        });
    }
}
