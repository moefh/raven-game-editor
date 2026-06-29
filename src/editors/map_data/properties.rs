use crate::app::WindowContext;
use crate::data_asset::{MapData, Tileset, DataAssetId, AssetIdList, AssetList};
use super::super::AssetEditorBase;

fn resize_map(map_data: &mut MapData, new_w: u32, new_h: u32, new_para_w: u32, new_para_h: u32, new_tile: u8) {
    fn resize_tiles(tiles: &mut Vec<u8>, old_w: usize, old_h: usize, new_w: usize, new_h: usize, new_tile: u8) {
        for y in 0..usize::min(old_h, new_h) {
            if new_w < old_w {
                let start = new_w * y + new_w;
                let len = old_w - new_w;
                tiles.drain(start .. start + len);
            } else if new_w > old_w {
                let start = new_w * y + old_w;
                let len = new_w - old_w;
                tiles.splice(start .. start, std::iter::repeat_n(new_tile, len));
            }
        }
        if new_h != old_h {
            tiles.resize(new_w * new_h, new_tile);
        }
    }

    let new_width = new_w as usize;
    let new_height = new_h as usize;
    let old_width = map_data.width as usize;
    let old_height = map_data.height as usize;
    resize_tiles(&mut map_data.fg_tiles, old_width, old_height, new_width, new_height, new_tile);
    resize_tiles(&mut map_data.bg_tiles, old_width, old_height, new_width, new_height, new_tile);
    resize_tiles(&mut map_data.fx_tiles, old_width, old_height, new_width, new_height, new_tile);
    map_data.width = new_w;
    map_data.height = new_h;

    let new_para_width = new_para_w as usize;
    let new_para_height = new_para_h as usize;
    let old_para_width = map_data.para_width as usize;
    let old_para_height = map_data.para_height as usize;
    resize_tiles(&mut map_data.para_tiles, old_para_width, old_para_height, new_para_width, new_para_height, new_tile);
    map_data.para_width = new_para_w;
    map_data.para_height = new_para_h;
}

pub struct PropertiesDialog {
    pub open: bool,
    pub name: String,
    pub tileset_id: DataAssetId,
    pub width: u32,
    pub height: u32,
    pub para_width: u32,
    pub para_height: u32,
    pub new_tile: u8,
    pub resized: bool,
}

impl PropertiesDialog {
    pub fn new(tileset_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
            tileset_id,
            width: 0,
            height: 0,
            para_width: 0,
            para_height: 0,
            new_tile: 0,
            resized: false,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_map_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, map_data: &MapData, new_tile: u8) {
        self.name.clear();
        self.name.push_str(&map_data.asset.name);
        self.tileset_id = map_data.tileset_id;
        self.width = map_data.width;
        self.height = map_data.height;
        self.para_width = map_data.para_width;
        self.para_height = map_data.para_height;
        self.new_tile = new_tile;
        self.resized = false;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, map_data: &mut MapData) -> bool {
        if self.width < self.para_width || self.height < self.para_height {
            wc.open_message_box("Invalid Size", "The parallax must be smaller or the same size as the foreground.");
            self.resized = false;
            return false;
        }

        map_data.asset.name.clear();
        map_data.asset.name.push_str(&self.name);
        map_data.tileset_id = self.tileset_id;

        let width = self.width;
        let height = self.height;
        let para_width = self.para_width;
        let para_height = self.para_height;
        if width != map_data.width || height != map_data.height || para_width != map_data.para_width || para_height != map_data.para_height {
            resize_map(map_data, width, height, para_width, para_height, self.new_tile);
            self.resized = true;
        } else {
            self.resized = false;
        }
        true
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if ! self.open { return; }

        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Map Properties", |ui, wc| {
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
                                for tileset_id in tileset_ids.iter() {
                                    if let Some(tileset) = tilesets.get(tileset_id) {
                                        ui.selectable_value(&mut self.tileset_id, tileset.asset.id, &tileset.asset.name);
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

                        ui.label("Parallax Width:");
                        ui.add(egui::Slider::new(&mut self.para_width, 0..=512).step_by(1.0));
                        ui.end_row();

                        ui.label("Parallax Height:");
                        ui.add(egui::Slider::new(&mut self.para_height, 0..=512).step_by(1.0));
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
            wc.set_dialog_open(Self::id(), self.open);
        }
    }
}
