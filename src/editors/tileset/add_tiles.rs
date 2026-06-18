use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::{
    AssetList,
    Tileset,
    MapData,
};
use super::super::AssetEditorBase;

pub enum AddTilesAction {
    Insert,
    Append,
}

pub struct AddTilesDialog {
    pub open: bool,
    pub action: AddTilesAction,
    pub num_tiles: u32,
    pub sel_tile: u32,
    pub clear_color: u8,
    confirmed: bool,
}

impl AddTilesDialog {
    pub fn new() -> Self {
        AddTilesDialog {
            confirmed: false,
            open: false,
            action: AddTilesAction::Insert,
            num_tiles: 0,
            sel_tile: 0,
            clear_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_add_tiles")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, action: AddTilesAction, sel_tile: u32, clear_color: u8) {
        self.action = action;
        self.num_tiles = 1;
        self.sel_tile = sel_tile;
        self.clear_color = clear_color;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn fix_map(map_data: &mut MapData, tile_index: u8, num_tiles: u8) {
        fn add_hole(tiles: &mut [u8], tile_index: u8, num_tiles: u8) {
            for tile in tiles {
                if *tile >= tile_index {
                    *tile = (*tile).saturating_add(num_tiles);
                }
            }
        }
        add_hole(&mut map_data.fg_tiles, tile_index, num_tiles);
        add_hole(&mut map_data.bg_tiles, tile_index, num_tiles);
        add_hole(&mut map_data.para_tiles, tile_index, num_tiles);
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
        let old_num_tiles = tileset.num_tiles;
        tileset.resize(tileset.width, tileset.height, tileset.num_tiles + self.num_tiles, self.clear_color);
        if matches!(self.action, AddTilesAction::Insert) && self.sel_tile < old_num_tiles {
            let tile_size = (tileset.height * tileset.width) as usize;
            let src_start = self.sel_tile as usize * tile_size;
            let src_end = (tileset.num_tiles - self.num_tiles) as usize * tile_size;
            let dst_start = (self.sel_tile + self.num_tiles) as usize * tile_size;
            tileset.data.copy_within(src_start..src_end, dst_start);
            tileset.data[src_start..dst_start].fill(self.clear_color);
            self.fix_maps(maps, tileset);
        }
        self.confirmed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset, maps: &mut AssetList<MapData>) -> bool {
        let title = match self.action {
            AddTilesAction::Insert => { "Insert Tiles" }
            AddTilesAction::Append => { "Append Tiles" }
        };
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, title, |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_add_tiles_grid", tileset.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Num tiles:");
                        let max = 255u32.saturating_sub(tileset.num_tiles);
                        if max == 0 {
                            ui.label("(max tiles reached)");
                        } else {
                            ui.add(egui::Slider::new(&mut self.num_tiles, 1..=max));
                        }
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
