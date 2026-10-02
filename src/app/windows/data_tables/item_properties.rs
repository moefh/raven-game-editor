use crate::data_asset::{
    self,
    Sprite,
    AssetIdCollection,
    AssetList,
    DataAssetId,
    DataAssetStore,
};

use super::super::{
    AppWindowBase,
    WindowContext,
};
use super::super::super::editors::DataTableType;

pub struct ItemPropertiesDialog {
    pub item_changed: bool,
    pub open: bool,
    pub item_index: usize,
    id: egui::Id,
    table_type: DataTableType,
    name: String,
    sprite_id: Option<DataAssetId>,
    sorted_sprite_ids: Vec<DataAssetId>,
}

impl ItemPropertiesDialog {
    pub fn new() -> Self {
        ItemPropertiesDialog {
            id: egui::Id::new("dlg_data_asset_item_table_properties"),
            open: false,
            item_changed: false,
            item_index: 0,
            table_type: DataTableType::Upgrades,
            name: String::new(),
            sprite_id: None,
            sorted_sprite_ids: Vec::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, table_type: DataTableType, item_index: usize, store: &DataAssetStore) {
        self.table_type = table_type;
        self.item_index = item_index;
        if let Some(item) = table_type.get_store_item(store, item_index) {
            self.name.replace_range(.., &item.name);
            self.sprite_id = Some(item.sprite_id);
        } else {
            self.name.clear();
            self.sprite_id = None;
        }
        self.sorted_sprite_ids.clear();
        self.open = true;
        wc.set_dialog_open(self.id, self.open);
    }

    fn confirm(&mut self, store: &mut DataAssetStore) {
        if let Some(item) = self.table_type.get_store_item_mut(store, self.item_index) {
            item.name.replace_range(.., &self.name);
            if let Some(sprite_id) = self.sprite_id {
                item.sprite_id = sprite_id;
            }
        }
    }

    fn sort_ids(&mut self, asset_ids: &AssetIdCollection, sprites: &AssetList<Sprite>) {
        if self.sorted_sprite_ids.is_empty() {
            asset_ids.sprites.copy_to(&mut self.sorted_sprite_ids);
            data_asset::utils::sort_asset_ids_by_name(&mut self.sorted_sprite_ids, sprites);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> bool {
        if ! self.open { return false; }
        self.sort_ids(&store.asset_ids, &store.assets.sprites);

        if AppWindowBase::show_dialog_window(wc, self.id, 300.0, "Item Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new("project_table_item_properties_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        if let Some(cur_sprite_id) = &mut self.sprite_id {
                            ui.label("Sprite:");
                            let cur_sprite_name = if let Some(sprite) = store.assets.sprites.get(cur_sprite_id) {
                                &sprite.asset.name
                            } else {
                                "??"
                            };
                            egui::ComboBox::from_id_salt("project_table_item_properties_sprite_combo")
                                .selected_text(cur_sprite_name)
                                .show_ui(ui, |ui| {
                                    for sprite_id in &self.sorted_sprite_ids {
                                        if let Some(sprite) = store.assets.sprites.get(sprite_id) {
                                            ui.selectable_value(cur_sprite_id, sprite.asset.id, &sprite.asset.name);
                                        }
                                    }
                                });
                            ui.end_row();
                        }
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(store);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.id, self.open);
        }
        if self.item_changed {
            self.item_changed = false;
            true
        } else {
            false
        }
    }
}
