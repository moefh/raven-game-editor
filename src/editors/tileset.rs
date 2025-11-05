use crate::IMAGES;
use crate::data_asset::{Tileset, DataAssetId};

pub struct TilesetEditor {
    pub asset: super::DataAssetEditor,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, tileset: &mut Tileset) {
        let title = format!("{} - Tileset", tileset.asset.name);
        let window = super::create_editor_window(self.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut tileset.asset.name);
                ui.add(egui::Image::new(IMAGES.tileset).max_width(32.0));
            });
        });
    }
}
