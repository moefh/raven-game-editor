use crate::data_asset::{
    MapData,
};

use super::super::{
    resize_map_tile_change,
    AssetEditorBase,
    WindowContext,
};

pub struct TileChangePropertiesDialog {
    dlg_id: egui::Id,
    pub open: bool,
    pub tile_change_index: usize,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl TileChangePropertiesDialog {
    pub fn new() -> Self {
        TileChangePropertiesDialog {
            dlg_id: egui::Id::unique("dlg_map_data_tile_change_properties"),
            open: false,
            tile_change_index: 0,
            name: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, map_data: &MapData, tile_change_index: usize) {
        if let Some(change) = map_data.tile_changes.get(tile_change_index) {
            self.name.replace_range(.., &change.name);
            self.x = change.x;
            self.y = change.y;
            self.width = change.width;
            self.height = change.height;
            self.tile_change_index = tile_change_index;
            self.open = true;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }

    fn confirm(&mut self, wc: &mut WindowContext, map_data: &mut MapData) -> bool {
        if self.width == 0 || self.x + self.width > map_data.width {
            wc.open_message_box(
                "Invalid Width",
                format!("The region with must be between 1 and {}", map_data.width.saturating_sub(self.x))
            );
            return false;
        }
        if self.height == 0 || self.y + self.height > map_data.height {
            wc.open_message_box(
                "Invalid Height",
                format!("The region height must be between 1 and {}", map_data.height.saturating_sub(self.y))
            );
            return false;
        }

        if let Some(change) = map_data.tile_changes.get_mut(self.tile_change_index) {
            change.name.replace_range(.., &self.name);
            resize_map_tile_change(change, self.width, self.height);
            change.x = self.x;
            change.y = self.y;
        }
        true
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData) {
        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 300.0, "Map Tile Change Properties", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_tile_change_prop_grid", map_data.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("X:");
                        ui.add(egui::Slider::new(&mut self.x, 0..=map_data.width.saturating_sub(1)));
                        ui.end_row();

                        ui.label("Y:");
                        ui.add(egui::Slider::new(&mut self.y, 0..=map_data.height.saturating_sub(1)));
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1..=map_data.width.saturating_sub(self.x)));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1..=map_data.height.saturating_sub(self.y)));
                        ui.end_row();

                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() && self.confirm(wc, map_data) {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }
}
