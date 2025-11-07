use crate::app::WindowContext;
use crate::IMAGES;
use crate::data_asset::{ModData, DataAssetId, GenericAsset};

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

    pub fn show(&mut self, wc: &WindowContext, mod_data: &mut ModData) {
        let title = format!("{} - Mod", mod_data.asset.name);
        let window = super::create_editor_window(mod_data.asset.id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", mod_data.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Mod", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", mod_data.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", mod_data.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add(egui::Image::new(IMAGES.mod_data).max_width(32.0));
            });
        });
    }
}
