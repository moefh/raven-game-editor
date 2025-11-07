use crate::app::WindowContext;
use crate::IMAGES;
use crate::data_asset::{PropFont, DataAssetId};

pub struct PropFontEditor {
    pub asset: super::DataAssetEditor,
}

impl PropFontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PropFontEditor {
            asset: super::DataAssetEditor {
                id,
                open,
            },
        }
    }

    pub fn show(&mut self, wc: &WindowContext, prop_font: &mut PropFont) {
        let title = format!("{} - Prop Font", prop_font.asset.name);
        let window = super::create_editor_window(prop_font.asset.id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                ui.text_edit_singleline(&mut prop_font.asset.name);
                ui.add(
                    egui::Image::new(IMAGES.prop_font).max_width(32.0)
                );
            });
        });
    }
}
