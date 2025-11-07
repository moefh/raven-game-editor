use crate::app::WindowContext;
use crate::IMAGES;
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId};

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
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut map_data.asset.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Tileset:");
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
                });
                ui.add(
                    egui::Image::new(IMAGES.map_data).max_width(32.0)
                );
            });
        });
    }
}
