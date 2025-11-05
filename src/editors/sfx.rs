use crate::IMAGES;
use crate::data_asset::{Sfx, DataAssetId};

pub struct SfxEditor {
    pub asset: super::DataAssetEditor,
}

impl SfxEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SfxEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, sfx: &mut Sfx) {
        let title = format!("{} - Sfx", sfx.asset.name);
        let window = super::create_editor_window(sfx.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut sfx.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.sfx).max_width(32.0)
                );
            });
        });
    }
}
