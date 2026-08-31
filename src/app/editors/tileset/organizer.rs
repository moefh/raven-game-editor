use crate::data_asset::Tileset;

use super::super::{
    AssetEditorBase,
    WindowContext,
    EditorAction,
};
use super::super::widgets::{
    TILE_SIZE,
    ImageOrganizerWidget,
};

pub struct OrganizerDialog {
    pub dlg_id: egui::Id,
    pub image_changed: bool,
    pub open: bool,
    pub tiles: ImageOrganizerWidget,
}

impl OrganizerDialog {
    pub fn new() -> Self {
        OrganizerDialog {
            dlg_id: egui::Id::new("dlg_tileset_organizer"),
            image_changed: false,
            open: false,
            tiles: ImageOrganizerWidget::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, tileset: &Tileset) {
        self.tiles.reset(tileset.num_tiles);
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        if tileset.num_tiles == self.tiles.indices.len() as u32 {
            let old = tileset.data.clone();
            let image_len = (tileset.width * tileset.height) as usize;
            for (to, &from) in self.tiles.indices.iter().enumerate() {
                let from = from as usize;
                tileset.data[to*image_len..(to+1)*image_len].copy_from_slice(&old[from*image_len..(from+1)*image_len]);
            }
            wc.add_editor_action(EditorAction::TilesetTilesShuffled {
                tileset_id: tileset.asset.id,
                shuffle: std::mem::take(&mut self.tiles.indices)
            });
        }
        self.image_changed = true;
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        let zoom = wc.settings.tile_picker_popup_zoom as f32 / 100.0;
        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, zoom * TILE_SIZE * 16.0 + 100.0, "Organize Tiles", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                self.tiles.show(ui, wc, tileset);
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(wc, tileset);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
