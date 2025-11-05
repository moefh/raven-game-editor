use crate::IMAGES;
use crate::data_asset::{ModData, DataAssetId};

pub struct ModDataEditor {
    pub asset: super::DataAssetEditor,
}

impl ModDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        ModDataEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, mod_data: &mut ModData) {
        let title = format!("{} - Mod", mod_data.asset.name);
        let window = super::create_editor_window(mod_data.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut mod_data.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.mod_data).max_width(32.0)
                );
            });
        });
    }
}
