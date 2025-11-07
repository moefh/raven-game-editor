use crate::app::WindowContext;
use crate::IMAGES;
use crate::data_asset::{PropFont, DataAssetId, GenericAsset};

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
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", prop_font.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Proportional Font", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", prop_font.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", prop_font.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add(egui::Image::new(IMAGES.prop_font).max_width(32.0));
            });
        });
    }
}
