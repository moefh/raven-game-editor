use crate::data_asset::DataAssetStore;

use super::super::{
    AppWindowBase,
    WindowContext,
};
use super::super::super::editors::DataTableType;

pub struct EffectPropertiesDialog {
    pub effect_changed: bool,
    pub open: bool,
    pub effect_index: usize,
    id: egui::Id,
    table_type: DataTableType,
    name: String,
}

impl EffectPropertiesDialog {
    pub fn new() -> Self {
        EffectPropertiesDialog {
            id: egui::Id::new("dlg_data_asset_effect_table_properties"),
            open: false,
            effect_changed: false,
            effect_index: 0,
            table_type: DataTableType::Upgrades,
            name: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, table_type: DataTableType, effect_index: usize, store: &DataAssetStore) {
        self.table_type = table_type;
        self.effect_index = effect_index;
        if let Some(effect_name) = table_type.get_store_effect(store, effect_index) {
            self.name.replace_range(.., effect_name);
        } else {
            self.name.clear();
        }
        self.open = true;
        wc.set_dialog_open(self.id, self.open);
    }

    fn confirm(&mut self, store: &mut DataAssetStore) {
        if let Some(effect_name) = self.table_type.get_store_effect_mut(store, self.effect_index) {
            effect_name.replace_range(.., &self.name);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> bool {
        if ! self.open { return false; }

        if AppWindowBase::show_dialog_window(wc, self.id, 300.0, "Effect Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new("project_table_effect_properties_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
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
        if self.effect_changed {
            self.effect_changed = false;
            true
        } else {
            false
        }
    }
}
