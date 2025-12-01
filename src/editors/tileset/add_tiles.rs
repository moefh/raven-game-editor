use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::Tileset;

pub enum AddTilesAction {
    Insert,
    Append,
}

pub struct AddTilesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub action: AddTilesAction,
    pub num_tiles: u32,
    pub sel_tile: u32,
    pub sel_color: u8,
}

impl AddTilesDialog {
    pub fn new() -> Self {
        AddTilesDialog {
            image_changed: false,
            open: false,
            action: AddTilesAction::Insert,
            num_tiles: 0,
            sel_tile: 0,
            sel_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_add_tiles")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, action: AddTilesAction, sel_tile: u32, sel_color: u8) {
        self.action = action;
        self.num_tiles = 1;
        self.sel_tile = sel_tile;
        self.sel_color = sel_color;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, tileset: &mut Tileset) {
        let image = ImageCollection::from_asset(tileset);
        image.resize(tileset.width, tileset.height, tileset.num_tiles + self.num_tiles, &mut tileset.data, self.sel_color);
        if matches!(self.action, AddTilesAction::Insert) && self.sel_tile < tileset.num_tiles {
            let src_top = self.sel_tile * tileset.height;
            let dst_top = (self.sel_tile + self.num_tiles) * tileset.height;
            let row_len = tileset.width as usize;
            let mut src_row = vec![0; row_len];
            let mut dst_row = vec![0; row_len];
            let num_new_rows = (tileset.num_tiles - self.sel_tile) * tileset.height;
            for y in (0..num_new_rows).rev() {
                let src = ((src_top + y) * tileset.width) as usize;
                let dst = ((dst_top + y) * tileset.width) as usize;
                src_row.copy_from_slice(&tileset.data[src..src+row_len]);
                dst_row.copy_from_slice(&tileset.data[dst..dst+row_len]);
                tileset.data[src..src+row_len].copy_from_slice(&dst_row);
                tileset.data[dst..dst+row_len].copy_from_slice(&src_row);
            }
        }
        tileset.num_tiles += self.num_tiles;
        self.image_changed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                match self.action {
                    AddTilesAction::Insert => { ui.heading("Insert Tiles"); }
                    AddTilesAction::Append => { ui.heading("Append Tiles"); }
                }
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_add_tiles_grid", tileset.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Num tiles:");
                            ui.add(egui::Slider::new(&mut self.num_tiles, 1..=16));
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(tileset);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
