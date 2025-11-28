use crate::app::WindowContext;
use crate::data_asset::{MapData, Tileset, DataAssetId, AssetIdList, AssetList};

fn resize_map(map_data: &mut MapData, new_w: u32, new_h: u32, new_bg_w: u32, new_bg_h: u32, new_tile: u8) {
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

    let new_w = new_w as usize;
    let new_h = new_h as usize;
    let old_w = map_data.width as usize;
    let old_h = map_data.height as usize;
    resize_tiles(&mut map_data.fg_tiles, old_w, old_h, new_w, new_h, new_tile);
    resize_tiles(&mut map_data.clip_tiles, old_w, old_h, new_w, new_h, new_tile);
    resize_tiles(&mut map_data.fx_tiles, old_w, old_h, new_w, new_h, new_tile);

    let new_bg_w = new_bg_w as usize;
    let new_bg_h = new_bg_h as usize;
    let old_bg_w = map_data.bg_width as usize;
    let old_bg_h = map_data.bg_height as usize;
    resize_tiles(&mut map_data.bg_tiles, old_bg_w, old_bg_h, new_bg_w, new_bg_h, new_tile);
}

pub struct PropertiesDialog {
    pub open: bool,
    pub name: String,
    pub tileset_id: DataAssetId,
    pub width: f32,
    pub height: f32,
    pub bg_width: f32,
    pub bg_height: f32,
    pub new_tile: u8,
    pub resized: bool,
}

impl PropertiesDialog {
    pub fn new(tileset_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
            tileset_id,
            width: 0.0,
            height: 0.0,
            bg_width: 0.0,
            bg_height: 0.0,
            new_tile: 0,
            resized: false,
        }
    }

    pub fn set_open(&mut self, map_data: &MapData, new_tile: u8) {
        self.name.clear();
        self.name.push_str(&map_data.asset.name);
        self.tileset_id = map_data.tileset_id;
        self.width = map_data.width as f32;
        self.height = map_data.height as f32;
        self.bg_width = map_data.bg_width as f32;
        self.bg_height = map_data.bg_height as f32;
        self.new_tile = new_tile;
        self.resized = false;
        self.open = true;
    }

    fn confirm(&mut self, wc: &mut WindowContext, map_data: &mut MapData) -> bool {
        if self.width < self.bg_width || self.height < self.bg_height {
            wc.dialogs.open_message_box("Invalid Size", "The background must be smaller or the same size as the foreground.");
            self.resized = false;
            return false;
        }

        map_data.asset.name.clear();
        map_data.asset.name.push_str(&self.name);
        map_data.tileset_id = self.tileset_id;

        let width = self.width as u32;
        let height = self.height as u32;
        let bg_width = self.bg_width as u32;
        let bg_height = self.bg_height as u32;
        if width != map_data.width || height != map_data.height || bg_width != map_data.bg_width || bg_height != map_data.bg_height {
            resize_map(map_data, width, height, bg_width, bg_height, self.new_tile);
            map_data.width = width;
            map_data.height = height;
            map_data.bg_width = bg_width;
            map_data.bg_height = bg_height;
            self.resized = true;
        } else {
            self.resized = false;
        }
        true
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if ! self.open { return; }

        if egui::Modal::new(egui::Id::new("dlg_map_data_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(350.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Map Properties");
                ui.separator();

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
                            ui.add(egui::Slider::new(&mut self.width, 1.0..=512.0).step_by(1.0));
                            ui.end_row();

                            ui.label("Height:");
                            ui.add(egui::Slider::new(&mut self.height, 1.0..=512.0).step_by(1.0));
                            ui.end_row();

                            ui.label("BG Width:");
                            ui.add(egui::Slider::new(&mut self.bg_width, 1.0..=512.0).step_by(1.0));
                            ui.end_row();

                            ui.label("BG Height:");
                            ui.add(egui::Slider::new(&mut self.bg_height, 1.0..=512.0).step_by(1.0));
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
            });
        }).should_close() {
            self.open = false;
        }
    }
}
