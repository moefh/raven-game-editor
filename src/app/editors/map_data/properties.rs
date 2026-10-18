use crate::data_asset::{
    self,
    DataAssetId,
    AssetIdCollection,
    AssetList,
    MapData,
    Tileset,
    TileAnimation,
};
use super::super::{
    resize_map_tiles,
    AssetEditorBase,
    WindowContext,
};

fn resize_map(map_data: &mut MapData, new_w: u32, new_h: u32, new_has_bb: bool, new_tile: u8) {
    resize_map_tiles(&mut map_data.fg_tiles, map_data.width, map_data.height, new_w, new_h, new_tile);
    resize_map_tiles(&mut map_data.bg_tiles, map_data.width, map_data.height, new_w, new_h, new_tile);
    resize_map_tiles(&mut map_data.fx_tiles, map_data.width, map_data.height, new_w, new_h, new_tile);
    if new_has_bb {
        let (old_w, old_h) = if map_data.has_bb { (map_data.width, map_data.height) } else { (0, 0) };
        resize_map_tiles(&mut map_data.bb_tiles, old_w, old_h, new_w, new_h, new_tile);
    } else {
        map_data.bb_tiles.clear();
    }
    map_data.width = new_w;
    map_data.height = new_h;
    map_data.has_bb = new_has_bb;
}

pub struct PropertiesDialog {
    pub open: bool,
    pub dlg_id: egui::Id,
    pub name: String,
    pub tileset_id: DataAssetId,
    pub tile_anim_id: Option<DataAssetId>,
    pub width: u32,
    pub height: u32,
    pub has_bb: bool,
    pub new_tile: u8,
    pub resized: bool,
    pub changed_tileset: bool,
    sorted_tileset_ids: Vec<DataAssetId>,
    sorted_tile_anim_ids: Vec<DataAssetId>,
}

impl PropertiesDialog {
    pub fn new(tileset_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            dlg_id: egui::Id::unique("dlg_map_properties"),
            name: String::new(),
            tileset_id,
            tile_anim_id: None,
            width: 0,
            height: 0,
            has_bb: false,
            new_tile: 0,
            resized: false,
            changed_tileset: false,
            sorted_tileset_ids: Vec::new(),
            sorted_tile_anim_ids: Vec::new(),
        }
    }

    pub fn set_open(
        &mut self,
        wc: &mut WindowContext,
        map_data: &MapData,
        new_tile: u8,
    ) {
        self.name.replace_range(.., &map_data.asset.name);
        self.tileset_id = map_data.tileset_id;
        self.tile_anim_id = map_data.tile_anim_id;
        self.width = map_data.width;
        self.height = map_data.height;
        self.has_bb = map_data.has_bb;
        self.new_tile = new_tile;
        self.resized = false;
        self.changed_tileset = false;
        self.sorted_tileset_ids.clear();
        self.sorted_tile_anim_ids.clear();
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, _wc: &mut WindowContext, map_data: &mut MapData) -> bool {
        map_data.asset.name.replace_range(.., &self.name);

        self.changed_tileset = map_data.tileset_id != self.tileset_id;
        map_data.tileset_id = self.tileset_id;
        map_data.tile_anim_id = self.tile_anim_id;

        if self.width != map_data.width || self.height != map_data.height || self.has_bb != map_data.has_bb {
            resize_map(map_data, self.width, self.height, self.has_bb, self.new_tile);
            self.resized = true;
        } else {
            self.resized = false;
        }
        true
    }

    fn sort_ids(&mut self, asset_ids: &AssetIdCollection, tilesets: &AssetList<Tileset>, tile_anims: &AssetList<TileAnimation>) {
        if self.sorted_tileset_ids.is_empty() {
            asset_ids.tilesets.copy_to(&mut self.sorted_tileset_ids);
            data_asset::utils::sort_asset_ids_by_name(&mut self.sorted_tileset_ids, tilesets);
        }
        if self.sorted_tile_anim_ids.is_empty() {
            asset_ids.tile_anims.copy_to(&mut self.sorted_tile_anim_ids);
            data_asset::utils::sort_asset_ids_by_name(&mut self.sorted_tile_anim_ids, tile_anims);
        }
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        map_data: &mut MapData,
        asset_ids: &AssetIdCollection,
        tilesets: &AssetList<Tileset>,
        tile_anims: &AssetList<TileAnimation>
    ) {
        if ! self.open { return; }
        self.sort_ids(asset_ids, tilesets, tile_anims);

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 350.0, "Map Properties", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", map_data.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Tileset:");
                        let cur_tileset_name = if let Some(cur_tileset) = tilesets.get(&self.tileset_id) {
                            &cur_tileset.asset.name
                        } else {
                            "??"
                        };
                        egui::ComboBox::from_id_salt(format!("map_editor_tileset_combo_{}", map_data.asset.id))
                            .selected_text(cur_tileset_name)
                            .show_ui(ui, |ui| {
                                for tileset_id in &self.sorted_tileset_ids {
                                    if let Some(tileset) = tilesets.get(tileset_id) {
                                        ui.selectable_value(&mut self.tileset_id, tileset.asset.id, &tileset.asset.name);
                                    }
                                }
                            });
                        ui.end_row();

                        ui.label("Tile Animation:");
                        let cur_tanim_name = if let Some(cur_tanim) = self.tile_anim_id.and_then(|tanim_id| tile_anims.get(&tanim_id)) {
                            &cur_tanim.asset.name
                        } else {
                            "(no animation)"
                        };
                        egui::ComboBox::from_id_salt(format!("map_editor_tile_anim_combo_{}", map_data.asset.id))
                            .selected_text(cur_tanim_name)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.tile_anim_id, None, "(no animation)");
                                for tile_anim_id in &self.sorted_tile_anim_ids {
                                    if let Some(tanim) = tile_anims.get(tile_anim_id) {
                                        ui.selectable_value(&mut self.tile_anim_id, Some(tanim.asset.id), &tanim.asset.name);
                                    }
                                }
                            });
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1..=512).step_by(1.0));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1..=512).step_by(1.0));
                        ui.end_row();

                        ui.label("2nd background:");
                        ui.checkbox(&mut self.has_bb, "");
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
