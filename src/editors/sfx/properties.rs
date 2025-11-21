use crate::app::WindowContext;
use crate::data_asset::Sfx;

pub struct PropertiesDialog {
    pub open: bool,
    pub name: String,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
        }
    }

    pub fn set_open(&mut self, sfx: &Sfx) {
        self.name.clear();
        self.name.push_str(&sfx.asset.name);
        self.open = true;
    }

    fn confirm(&mut self, sfx: &mut Sfx) {
        sfx.asset.name.clear();
        sfx.asset.name.push_str(&self.name);
    }

    pub fn show(&mut self, wc: &WindowContext, sfx: &mut Sfx) {
        if egui::Modal::new(egui::Id::new("dlg_sfx_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Sfx Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", sfx.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(sfx);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}
