use crate::app::WindowContext;
use crate::data_asset::World;
use super::super::AssetEditorBase;

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

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_world_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, world: &World) {
        self.name.clear();
        self.name.push_str(&world.asset.name);
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, world: &mut World) {
        world.asset.name.clear();
        world.asset.name.push_str(&self.name);
    }

    pub fn show(&mut self, wc: &mut WindowContext, world: &mut World) {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 300.0, "World Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", world.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(world);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
    }
}
