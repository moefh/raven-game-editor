use crate::image::{
    ImageCollectionIO,
    ImagePixelsCollection,
};
use crate::data_asset::Tileset;

use super::super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
    SelectImageLocation,
};

pub struct ExportSelectedTilesDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub dlg_id_insert_location_combo: egui::Id,
    pub selected_tile: u32,
    pub start_tile_location: SelectImageLocation,
    pub num_tiles: u32,
    pub num_tiles_x: u32,
    pub export_sys_dlg_id: String,
}

impl ExportSelectedTilesDialog {
    pub fn new() -> Self {
        ExportSelectedTilesDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_tileset_export_sel_tiles"),
            dlg_id_insert_location_combo: egui::Id::new("dlg_tileset_export_sel_tiles_combo"),
            selected_tile: 0,
            start_tile_location: SelectImageLocation::Selected,
            num_tiles: 1,
            num_tiles_x: 1,
            export_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, selected_tile: u32, tileset: &Tileset) {
        self.selected_tile = selected_tile;
        self.num_tiles = 1;
        self.num_tiles_x = 1;
        self.start_tile_location = SelectImageLocation::Selected;
        self.export_sys_dlg_id.replace_range(.., &format!("editor_{}_export_sel_tiles", tileset.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        let first_tile = match self.start_tile_location {
            SelectImageLocation::Start => { 0 }
            SelectImageLocation::Selected => { self.selected_tile }
        };
        if first_tile + self.num_tiles > tileset.num_tiles {
            wc.open_message_box("Error Exporting", "Invalid number of tiles to export");
            return false;
        }

        const TILE_LEN: usize = (Tileset::TILE_SIZE * Tileset::TILE_SIZE) as usize;
        let mut export_image = ImagePixelsCollection::new(tileset.width, tileset.height, self.num_tiles);
        let src_start = first_tile as usize * TILE_LEN;
        let copy_len = self.num_tiles as usize * TILE_LEN;
        export_image.data[0..copy_len].copy_from_slice(&tileset.data[src_start .. src_start+copy_len]);
        if let Err(e) = export_image.save_image_png(self.num_tiles_x).and_then(|data| file.write_data(data)) {
            wc.open_message_box("Error Exporting", format!("Error exporting selected tiles to {}:\n{}", file.filename(), e));
            false
        } else {
            true
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        if ! self.open { return; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.export_sys_dlg_id) &&
            self.confirm(file, wc, tileset) {
                self.close(wc);
                return;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 300.0, "Export Tileset", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_export_grid", tileset.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Start at:");
                        egui::ComboBox::from_id_salt(self.dlg_id_insert_location_combo)
                            .selected_text(self.start_tile_location.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.start_tile_location,
                                    SelectImageLocation::Start,
                                    SelectImageLocation::Start.text()
                                );
                                ui.selectable_value(
                                    &mut self.start_tile_location,
                                    SelectImageLocation::Selected,
                                    SelectImageLocation::Selected.text()
                                );
                            });
                        ui.end_row();

                        let max_total_tiles = match self.start_tile_location {
                            SelectImageLocation::Start => { tileset.num_tiles }
                            SelectImageLocation::Selected => { tileset.num_tiles.saturating_sub(self.selected_tile).max(1) }
                        };
                        ui.label("Total Tiles:");
                        ui.add(egui::Slider::new(&mut self.num_tiles, 1..=max_total_tiles));
                        ui.end_row();

                        ui.label("Horiz Tiles:");
                        ui.add(egui::Slider::new(&mut self.num_tiles_x, 1..=self.num_tiles));
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Save File").clicked() {
                    wc.sys_dialogs.save_file(
                        Some(wc.egui.window),
                        self.export_sys_dlg_id.clone(),
                        "tileset",
                        "Export Tileset",
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
    }
}
