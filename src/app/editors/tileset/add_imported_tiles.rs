use crate::image::{
    ImageCollection,
    ImageCollectionIO,
    ImageSlicingMethod,
    ImageLoadOptions,
    ImagePixelsCollection,
};
use crate::data_asset::Tileset;

use super::super::{
    AssetEditorBase,
    WindowContext,
    EditorAction,
    SysDialogResponse,
    SysDialogOpenFile,
    AddTileLocation,
};

pub struct AddImportedTilesDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub import_tileset_sys_dlg_id: String,
    pub load_options: ImageLoadOptions,
    pub add_tile_location: AddTileLocation,
    pub sel_tile: u32,
    pub clear_color: u8,
}

impl AddImportedTilesDialog {
    const DEFAULT_LOAD_OPTIONS: ImageLoadOptions = ImageLoadOptions {
        slicing_method: ImageSlicingMethod::by_size(Tileset::TILE_SIZE, Tileset::TILE_SIZE),
        border: 0,
        space_between: 0,
        zoom_x: 1,
        zoom_y: 1,
    };

    pub fn new() -> Self {
        AddImportedTilesDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_tileset_import"),
            load_options: Self::DEFAULT_LOAD_OPTIONS,
            add_tile_location: AddTileLocation::AtEnd,
            sel_tile: 0,
            clear_color: 0,
            import_tileset_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sel_tile: u32, clear_color: u8, tileset: &Tileset) {
        self.add_tile_location = AddTileLocation::AtEnd;
        self.sel_tile = sel_tile;
        self.clear_color = clear_color;
        self.load_options = Self::DEFAULT_LOAD_OPTIONS;
        self.import_tileset_sys_dlg_id.replace_range(.., &format!("editor_{}_import_tileset", tileset.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn create_empty_tiles(&mut self, wc: &mut WindowContext, num_tiles: u32, tileset: &mut Tileset) -> usize {
        let old_num_tiles = tileset.num_tiles;
        let insertion_point = match self.add_tile_location {
            AddTileLocation::BeforeSelected => { self.sel_tile.min(tileset.num_tiles) }
            AddTileLocation::AfterSelected => { (self.sel_tile + 1).min(tileset.num_tiles) }
            AddTileLocation::AtEnd => { tileset.num_tiles }
        };
        tileset.resize(tileset.width, tileset.height, tileset.num_tiles + num_tiles, self.clear_color);
        if insertion_point < old_num_tiles {
            let tile_size = (tileset.height * tileset.width) as usize;
            let src_start = insertion_point as usize * tile_size;
            let src_end = (tileset.num_tiles - num_tiles) as usize * tile_size;
            let dst_start = (insertion_point + num_tiles) as usize * tile_size;
            tileset.data.copy_within(src_start..src_end, dst_start);
            tileset.data[src_start..dst_start].fill(self.clear_color);
            let num_tiles_after_hole = old_num_tiles - insertion_point;
            if num_tiles_after_hole <= u8::MAX as u32 && insertion_point <= u8::MAX as u32 && num_tiles <= u8::MAX as u32 {
                wc.add_editor_action(EditorAction::TilesetTilesAdded {
                    tileset_id: tileset.asset.id,
                    hole_start: insertion_point as u8,
                    hole_size: num_tiles as u8,
                    num_tiles_after_hole: num_tiles_after_hole as u8,
                });
            }
        }
        insertion_point as usize * (tileset.height * tileset.width) as usize
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        let tiles = file.read_data().and_then(|data| {
            let mut tiles = ImagePixelsCollection::new(Tileset::TILE_SIZE, Tileset::TILE_SIZE, 1);
            tiles.load_image_png(&data, &self.load_options).map(|_| tiles)
        });
        match tiles {
            Ok(tiles) => {
                let num_tiles = tiles.num_items.min(255_u32.saturating_sub(tileset.num_tiles));
                if num_tiles == 0 {
                    wc.open_message_box(
                        "Error adding tiles",
                        "This tileset has the maximum number of tiles."
                    );
                } else {
                    let tile_len = (tileset.height * tileset.width) as usize;
                    let dst_start = self.create_empty_tiles(wc, num_tiles, tileset);
                    let dst_len = num_tiles as usize * tile_len;
                    tileset.data[dst_start..dst_len].copy_from_slice(&tiles.data[..dst_len]);
                }
                true
            }
            Err(e) => {
                wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error importing tileset",
                    "Error importing tileset file.\n\nConsult the log window for more information."
                );
                false
            }
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_tileset_sys_dlg_id) &&
            self.confirm(file, wc, tileset) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Import Tileset", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", tileset.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Zoom X:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_x, 1..=256));
                        ui.end_row();

                        ui.label("Zoom Y:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_y, 1..=256));
                        ui.end_row();

                        ui.label("Border:");
                        ui.add(egui::Slider::new(&mut self.load_options.border, 0..=32));
                        ui.end_row();

                        ui.label("Space between:");
                        ui.add(egui::Slider::new(&mut self.load_options.space_between, 0..=32));
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
                if ui.button("Open File").clicked() {
                    wc.sys_dialogs.open_file(
                        Some(wc.egui.window),
                        self.import_tileset_sys_dlg_id.clone(),
                        "tileset",
                        "Import Tileset",
                        &[
                            ("PNG files (*.png)", &["png"]),
                            ("All files (*.*)", &["*"]),
                        ]
                    );
                }
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.close(wc);
        }
        false
    }
}
