use crate::IMAGES;
use crate::data_asset::{Font, DataAssetId};

pub struct FontEditor {
    pub asset: super::DataAssetEditor,
}

impl FontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        FontEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, font: &mut Font) {
        let title = format!("{} - Font", font.asset.name);
        let window = super::create_editor_window(font.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut font.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.font).max_width(32.0)
                );
            });
        });
    }
}
