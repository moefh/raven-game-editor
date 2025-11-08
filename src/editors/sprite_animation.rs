use crate::IMAGES;
use crate::misc::WindowContext;
use crate::data_asset::{SpriteAnimation, DataAssetId, GenericAsset};

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

    pub fn show(&mut self, wc: &WindowContext, animation: &mut SpriteAnimation) {
        let title = format!("{} - Animation", animation.asset.name);
        let window = super::create_editor_window(animation.asset.id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", animation.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Animation", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                //...
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", animation.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", animation.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add(egui::Image::new(IMAGES.animation).max_width(32.0));
            });
        });
    }
}
