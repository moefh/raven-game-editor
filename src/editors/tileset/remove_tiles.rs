use crate::app::{
    WindowContext,
    EditorAction,
};
use crate::image::ImageCollection;
use crate::data_asset::Tileset;
use super::super::AssetEditorBase;

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
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, tileset: &mut Tileset, wc: &mut WindowContext) {
        if self.sel_tile + self.num_tiles < tileset.num_tiles {
            let tile_size = (tileset.height * tileset.width) as usize;
            let src_start = (self.sel_tile + self.num_tiles) as usize * tile_size;
            let src_end = tileset.num_tiles as usize * tile_size;
            let dst_start = self.sel_tile as usize * tile_size;
            tileset.data.copy_within(src_start..src_end, dst_start);
            if self.sel_tile <= u8::MAX as u32 && self.num_tiles <= u8::MAX as u32 {
                wc.add_editor_action(EditorAction::FixMapsAfterTilesRemoved {
                    tileset_id: tileset.asset.id,
                    tile_index: self.sel_tile as u8,
                    num_tiles: self.num_tiles as u8,
                });
            }
        }
        tileset.resize(tileset.width, tileset.height, tileset.num_tiles - self.num_tiles, 0);
        self.confirmed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Remove Tiles", |ui, wc| {
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
                    self.confirm(tileset, wc);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
        if self.confirmed {
            self.confirmed = false;
            true
        } else {
            false
        }
    }
}
