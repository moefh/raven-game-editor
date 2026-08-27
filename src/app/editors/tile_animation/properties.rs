use crate::data_asset::{
    DataAssetId,
    AssetList,
    AssetIdList,
    Tileset,
    TileAnimation,
};

use super::super::{
    AssetEditorBase,
    WindowContext,
};

pub struct PropertiesDialog {
    pub open: bool,
    pub parent_tileset_id: DataAssetId,
    pub anim_tileset_id: DataAssetId,
    pub name: String,
    dlg_id: egui::Id,
    prop_grid_id: String,
    parent_combo_id: String,
    anim_combo_id: String,
}

impl PropertiesDialog {
    pub fn new(tile_anim_id: DataAssetId, parent_tileset_id: DataAssetId, anim_tileset_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            parent_tileset_id,
            anim_tileset_id,
            name: String::new(),
            dlg_id: egui::Id::new("dlg_animation_properties"),
            prop_grid_id: format!("editor_panel_{}_prop_grid", tile_anim_id),
            parent_combo_id: format!("editor_panel_{}_parent_tileset", tile_anim_id),
            anim_combo_id: format!("editor_panel_{}_anim_tileset", tile_anim_id),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, tanim: &TileAnimation) {
        self.name.clear();
        self.name.push_str(&tanim.asset.name);
        self.parent_tileset_id = tanim.parent_tileset_id;
        self.anim_tileset_id = tanim.anim_tileset_id;
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, tanim: &mut TileAnimation) {
        tanim.asset.name.clear();
        tanim.asset.name.push_str(&self.name);
        tanim.parent_tileset_id = self.parent_tileset_id;
        tanim.anim_tileset_id = self.anim_tileset_id;
    }

    fn show_tileset_combo(
        ui: &mut egui::Ui,
        combo_id: &str,
        cur_tileset_id: &mut DataAssetId,
        tileset_ids: &AssetIdList,
        tilesets: &AssetList<Tileset>
    ) {
        let cur_tileset_name = if let Some(cur_tileset) = tilesets.get(cur_tileset_id) {
            &cur_tileset.asset.name
        } else {
            "??"
        };
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(cur_tileset_name)
            .show_ui(ui, |ui| {
                for tileset_id in tileset_ids.iter() {
                    if let Some(tileset) = tilesets.get(tileset_id) {
                        ui.selectable_value(cur_tileset_id, tileset.asset.id, &tileset.asset.name);
                    }
                }
            });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        tanim: &mut TileAnimation,
        tileset_ids: &AssetIdList,
        tilesets: &AssetList<Tileset>
    ) {
        if ! self.open { return; }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 450.0, "Animation Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(&self.prop_grid_id)
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Map tileset:");
                        Self::show_tileset_combo(ui, &self.parent_combo_id, &mut self.parent_tileset_id, tileset_ids, tilesets);
                        ui.end_row();

                        ui.label("Animation tileset:");
                        Self::show_tileset_combo(ui, &self.anim_combo_id, &mut self.anim_tileset_id, tileset_ids, tilesets);
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(tanim);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }
}
