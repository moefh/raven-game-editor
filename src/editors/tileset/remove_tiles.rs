use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::{
    AssetList,
    Tileset,
    MapData,
};

pub struct RemoveTilesDialog {
    pub confirmed: bool,
    pub open: bool,
    pub num_tiles: u32,
    pub max_tiles: u32,
    pub sel_tile: u32,
}

impl RemoveTilesDialog {
    pub fn new() -> Self {
        RemoveTilesDialog {
            confirmed: false,
            open: false,
            num_tiles: 0,
            max_tiles: 0,
            sel_tile: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_rm_tiles")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, tileset: &Tileset, sel_tile: u32) {
        if tileset.num_tiles <= 1 || tileset.num_tiles <= sel_tile { return; }
        self.max_tiles = (tileset.num_tiles - sel_tile).min(tileset.num_tiles - 1);
        self.num_tiles = 1;
        self.sel_tile = sel_tile;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn fix_map(map_data: &mut MapData, tile_index: u8, num_tiles: u8) {
        fn rm_hole(tiles: &mut [u8], tile_index: u8, num_tiles: u8) {
            for tile in tiles {
                if *tile >= tile_index + num_tiles {
                    *tile = (*tile).saturating_sub(num_tiles);
                }
            }
        }
        rm_hole(&mut map_data.fg_tiles, tile_index, num_tiles);
        rm_hole(&mut map_data.bg_tiles, tile_index, num_tiles);
        rm_hole(&mut map_data.para_tiles, tile_index, num_tiles);
    }

    fn fix_maps(&self, maps: &mut AssetList<MapData>, tileset: &Tileset) {
        if self.sel_tile >= 256 || self.num_tiles >= 256 {
            return;
        }
        let tile_index = self.sel_tile as u8;
        let num_tiles = self.num_tiles as u8;
        for map_data in maps.iter_mut() {
            if map_data.tileset_id == tileset.asset.id {
                Self::fix_map(map_data, tile_index, num_tiles);
            }
        }
    }

    fn confirm(&mut self, tileset: &mut Tileset, maps: &mut AssetList<MapData>) {
        if self.sel_tile + self.num_tiles < tileset.num_tiles {
            let tile_size = (tileset.height * tileset.width) as usize;
            let src_start = (self.sel_tile + self.num_tiles) as usize * tile_size;
            let src_end = tileset.num_tiles as usize * tile_size;
            let dst_start = self.sel_tile as usize * tile_size;
            tileset.data.copy_within(src_start..src_end, dst_start);
            self.fix_maps(maps, tileset);
        }
        tileset.resize(tileset.width, tileset.height, tileset.num_tiles - self.num_tiles, 0);
        self.confirmed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset, maps: &mut AssetList<MapData>) -> bool {
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Remove Tiles");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_add_tiles_grid", tileset.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Num tiles:");
                            ui.add(egui::Slider::new(&mut self.num_tiles, 1..=16.min(self.max_tiles)));
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(tileset, maps);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        if self.confirmed {
            self.confirmed = false;
            true
        } else {
            false
        }
    }
}
