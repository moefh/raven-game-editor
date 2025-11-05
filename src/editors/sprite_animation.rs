use crate::IMAGES;
use crate::data_asset::{SpriteAnimation, DataAssetId};

pub struct SpriteAnimationEditor {
    pub asset: super::DataAssetEditor,
}

impl SpriteAnimationEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteAnimationEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, window_space: egui::Rect, animation: &mut SpriteAnimation) {
        let title = format!("{} - Animation", animation.asset.name);
        let window = super::create_editor_window(animation.asset.id, &title, window_space);
        window.open(&mut self.asset.open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut animation.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.animation).max_width(32.0)
                );
            });
        });
    }
}
