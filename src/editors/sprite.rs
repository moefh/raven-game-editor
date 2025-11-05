use crate::IMAGES;
use crate::data_asset::{Sprite, DataAssetId};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, sprite: &mut Sprite) {
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(sprite.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut sprite.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.sprite).max_width(32.0)
                );
            });
        });
    }
}
