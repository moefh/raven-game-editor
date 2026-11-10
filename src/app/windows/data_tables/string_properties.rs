use crate::data_store::DataStore;

use super::super::{
    AppWindowBase,
    WindowContext,
};
use super::super::super::editors::DataTableReference;

pub struct StringPropertiesDialog {
    pub item_changed: bool,
    pub open: bool,
    pub item_index: usize,
    id: egui::Id,
    table_ref: DataTableReference,
    name: String,
    value: String,
}

impl StringPropertiesDialog {
    pub fn new() -> Self {
        StringPropertiesDialog {
            id: egui::Id::unique("dlg_data_asset_string_table_properties"),
            open: false,
            item_changed: false,
            item_index: 0,
            table_ref: DataTableReference::Upgrades,
            name: String::new(),
            value: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, table_ref: DataTableReference, item_index: usize, store: &DataStore) {
        self.table_ref = table_ref;
        self.item_index = item_index;
        if let Some(item) = table_ref.get_store_string(store, item_index) {
            self.name.replace_range(.., &item.name);
            self.value.replace_range(.., &item.value);
        } else {
            self.name.clear();
            self.value.clear();
        }
        self.open = true;
        wc.set_dialog_open(self.id, self.open);
    }

    fn confirm(&mut self, store: &mut DataStore) {
        if let Some(item) = self.table_ref.get_store_string_mut(store, self.item_index) {
            item.name.replace_range(.., &self.name);
            item.value.replace_range(.., &self.value);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataStore) -> bool {
        if ! self.open { return false; }

        if AppWindowBase::show_dialog_window(wc, self.id, 300.0, "String Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new("project_table_string_properties_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Value:");
                        ui.text_edit_singleline(&mut self.value);
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
        if self.item_changed {
            self.item_changed = false;
            true
        } else {
            false
        }
    }
}
