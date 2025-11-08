use crate::IMAGES;
use crate::app::WindowContext;
use crate::data_asset::{Sfx, DataAssetId, GenericAsset};

pub struct SfxEditor {
    pub asset: super::DataAssetEditor,
    state: super::widgets::SfxDisplayState,
}

impl SfxEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SfxEditor {
            asset: super::DataAssetEditor::new(id, open),
            state: super::widgets::SfxDisplayState::new(),
        }
    }

    pub fn show(&mut self, wc: &WindowContext, sfx: &mut Sfx) {
        let title = format!("{} - Sfx", sfx.asset.name);
        let window = super::create_editor_window(sfx.asset.id, &title, wc);

        let mut loop_start = sfx.loop_start as f32;
        let mut loop_end = (sfx.loop_start + sfx.loop_len) as f32;

        window.open(&mut self.asset.open).min_size([300.0, 150.0]).default_size([500.0, 200.0]).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", sfx.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Sfx", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", sfx.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", sfx.data_size()));
            });

            // loop start/end
            egui::SidePanel::left(format!("editor_panel_{}_left", sfx.asset.id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                egui::Grid::new(format!("editor_panel_{}_loop_grid", sfx.asset.id)).num_columns(2).show(ui, |ui| {
                    ui.label("Loop start:");
                    ui.add(egui::DragValue::new(&mut loop_start).speed(1.0).range(0.0..=sfx.samples.len() as f32));
                    ui.end_row();

                    ui.label("Loop end:");
                    ui.add(egui::DragValue::new(&mut loop_end).speed(1.0).range(loop_start..=sfx.samples.len() as f32));
                    ui.end_row();
                });
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                super::widgets::sfx_display(ui, &mut self.state, &sfx.samples, &mut loop_start, &mut loop_end);
            });
        });

        sfx.loop_start = loop_start as u32;
        sfx.loop_len = (loop_end - loop_start) as u32;
    }
}
