use crate::IMAGES;
use crate::misc::WindowContext;
use crate::data_asset::{Font, DataAssetId, GenericAsset};

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

    pub fn show(&mut self, wc: &WindowContext, font: &mut Font) {
        let asset_id = font.asset.id;
        let title = format!("{} - Font", font.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Font", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", font.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add(egui::Image::new(IMAGES.font).max_width(32.0));
            });
        });
    }
}
