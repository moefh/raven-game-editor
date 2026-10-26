use crate::data_store::{
    self,
    DataAssetId,
    AssetIdCollection,
    DataAssetList,
    Room,
    MapData,
    Tileset,
};
use super::super::{
    resize_map_tiles,
    AssetEditorBase,
    WindowContext,
    RoomSize,
};

const MIN_PARA_WIDTH: u32 = 20;
const MIN_PARA_HEIGHT: u32 = 15;

pub struct PropertiesDialog {
    pub open: bool,
    pub dlg_id: egui::Id,
    pub name: String,
    pub has_script: bool,
    pub para_tileset_id: Option<DataAssetId>,
    pub para_width: u32,
    pub para_height: u32,
    pub max_para_width: u32,
    pub max_para_height: u32,
    pub new_tile: u8,
    sorted_tileset_ids: Vec<DataAssetId>,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            open: false,
            dlg_id: egui::Id::unique("dlg_room_properties"),
            name: String::new(),
            has_script: false,
            para_tileset_id: None,
            para_width: 0,
            para_height: 0,
            max_para_width: 0,
            max_para_height: 0,
            new_tile: 0,
            sorted_tileset_ids: Vec::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, room: &Room, maps: &DataAssetList<MapData>, new_tile: u8) {
        self.name.clear();
        self.name.push_str(&room.asset.name);
        self.has_script = room.has_script;
        self.para_tileset_id = room.parallax.tileset_id;
        self.para_width = room.parallax.width;
        self.para_height = room.parallax.height;
        let room_size = RoomSize::from_room(room, maps);
        self.max_para_width = room_size.width.max(MIN_PARA_WIDTH);
        self.max_para_height = room_size.height.max(MIN_PARA_HEIGHT);
        self.new_tile = new_tile;
        self.sorted_tileset_ids.clear();
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, room: &mut Room) {
        room.asset.name.replace_range(.., &self.name);
        room.has_script = self.has_script;
        room.parallax.tileset_id = self.para_tileset_id;
        if self.para_tileset_id.is_none() {
            self.para_width = 0;
            self.para_height = 0;
        } else {
            self.para_width = self.para_width.clamp(MIN_PARA_WIDTH, self.max_para_width);
            self.para_height = self.para_height.clamp(MIN_PARA_HEIGHT, self.max_para_height);
        }
        if self.para_width != room.parallax.width || self.para_height != room.parallax.height {
            if self.para_width == 0 || self.para_height == 0 {
                room.parallax.tiles.clear();
            } else {
                resize_map_tiles(
                    &mut room.parallax.tiles,
                    room.parallax.width,
                    room.parallax.height,
                    self.para_width,
                    self.para_height,
                    self.new_tile
                );
            }
            room.parallax.width = self.para_width;
            room.parallax.height = self.para_height;
        }
    }

    fn sort_ids(&mut self, asset_ids: &AssetIdCollection, tilesets: &DataAssetList<Tileset>) {
        if self.sorted_tileset_ids.is_empty() {
            asset_ids.tilesets.copy_to(&mut self.sorted_tileset_ids);
            data_store::utils::sort_asset_ids_by_name(&mut self.sorted_tileset_ids, tilesets);
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, room: &mut Room, asset_ids: &AssetIdCollection, tilesets: &DataAssetList<Tileset>) {
        if ! self.open { return; }
        self.sort_ids(asset_ids, tilesets);

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 400.0, "Room Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", room.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Has script:");
                        ui.checkbox(&mut self.has_script, "");
                        ui.end_row();

                        ui.vertical(|ui| {
                            ui.add_space(2.0);
                            ui.label("Parallax:");
                        });
                        ui.vertical(|ui| {
                            ui.add_space(2.0);
                            egui::Grid::new(format!("editor_panel_{}_para_grid", room.asset.id))
                                .num_columns(2)
                                .show(ui, |ui| {
                                    ui.label("Tileset:");
                                    let cur_tileset_name = if let Some(cur_tileset) = self.para_tileset_id.and_then(|id| tilesets.get(&id)) {
                                        &cur_tileset.asset.name
                                    } else {
                                        "(none)"
                                    };
                                    egui::ComboBox::from_id_salt(format!("room_editor_tileset_combo_{}", room.asset.id))
                                        .selected_text(cur_tileset_name)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.para_tileset_id, None, "(none)");
                                            for tileset_id in &self.sorted_tileset_ids {
                                                if let Some(tileset) = tilesets.get(tileset_id) {
                                                    ui.selectable_value(&mut self.para_tileset_id, Some(tileset.asset.id), &tileset.asset.name);
                                                }
                                            }
                                        });
                                    ui.end_row();

                                    ui.label("Width:");
                                    if self.para_tileset_id.is_none() {
                                        ui.label("0");
                                    } else {
                                        ui.add(egui::Slider::new(&mut self.para_width, MIN_PARA_WIDTH..=self.max_para_width).step_by(1.0));
                                    }
                                    ui.end_row();

                                    ui.label("Height:");
                                    if self.para_tileset_id.is_none() {
                                        ui.label("0");
                                    } else {
                                        ui.add(egui::Slider::new(&mut self.para_height, MIN_PARA_HEIGHT..=self.max_para_height).step_by(1.0));
                                    }
                                    ui.end_row();
                                });
                            ui.end_row();
                        });
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(room);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
    }
}
