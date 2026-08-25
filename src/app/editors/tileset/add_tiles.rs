use crate::image::ImageCollection;
use crate::data_asset::Tileset;

use super::super::{
    AssetEditorBase,
    WindowContext,
    EditorAction,
    AddTileLocation,
};

pub struct AddTilesDialog {
    pub open: bool,
    pub add_tile_location: AddTileLocation,
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
            add_tile_location: AddTileLocation::AfterSelected,
            num_tiles: 0,
            sel_tile: 0,
            clear_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_add_tiles")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sel_tile: u32, clear_color: u8) {
        self.add_tile_location = AddTileLocation::AfterSelected;
        self.num_tiles = 1;
        self.sel_tile = sel_tile;
        self.clear_color = clear_color;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, tileset: &mut Tileset, wc: &mut WindowContext) {
        let old_num_tiles = tileset.num_tiles;
        let insertion_point = match self.add_tile_location {
            AddTileLocation::BeforeSelected => { self.sel_tile.min(tileset.num_tiles) }
            AddTileLocation::AfterSelected => { (self.sel_tile + 1).min(tileset.num_tiles) }
            AddTileLocation::AtEnd => { tileset.num_tiles }
        };
        tileset.resize(tileset.width, tileset.height, tileset.num_tiles + self.num_tiles, self.clear_color);
        let tile_size = (tileset.height * tileset.width) as usize;
        let src_start = insertion_point as usize * tile_size;
        let src_end = (tileset.num_tiles - self.num_tiles) as usize * tile_size;
        let dst_start = (insertion_point + self.num_tiles) as usize * tile_size;
        tileset.data.copy_within(src_start..src_end, dst_start);
        tileset.data[src_start..dst_start].fill(self.clear_color);
        let num_tiles_after_hole = old_num_tiles - insertion_point;
        if num_tiles_after_hole <= u8::MAX as u32 && insertion_point <= u8::MAX as u32 && self.num_tiles <= u8::MAX as u32 {
            wc.add_editor_action(EditorAction::TilesetTilesAdded {
                tileset_id: tileset.asset.id,
                hole_start: insertion_point as u8,
                hole_size: self.num_tiles as u8,
                num_tiles_after_hole: num_tiles_after_hole as u8,
            });
        }
        self.confirmed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 350.0, "Add Tiles", |ui, wc| {
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

                        ui.label("Insert at:");
                        egui::ComboBox::from_id_salt(format!("editor_panel_{}_insert_tile_at_combo", tileset.asset.id))
                            .selected_text(self.add_tile_location.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.add_tile_location,
                                    AddTileLocation::BeforeSelected,
                                    AddTileLocation::BeforeSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_tile_location,
                                    AddTileLocation::AfterSelected,
                                    AddTileLocation::AfterSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_tile_location,
                                    AddTileLocation::AtEnd,
                                    AddTileLocation::AtEnd.text()
                                );
                            });
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
