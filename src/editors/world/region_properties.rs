use crate::app::WindowContext;
use crate::data_asset::{
    World,
    WorldRegion,
};
use super::super::AssetEditorBase;

pub struct RegionPropertiesDialog {
    pub open: bool,
    pub region_index: usize,
    pub name: String,
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

impl RegionPropertiesDialog {
    pub fn new() -> Self {
        RegionPropertiesDialog {
            open: false,
            region_index: 0,
            name: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_world_region_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, world: &World, region_index: usize) {
        if let Some(region) = world.regions.get(region_index) {
            self.name.clear();
            self.name.push_str(&region.name);
            self.x = region.x;
            self.y = region.y;
            self.width = region.width;
            self.height = region.height;
            self.region_index = region_index;
            self.open = true;
            wc.set_dialog_open(Self::id(), self.open);
        }
    }

    fn confirm(&mut self, wc: &mut WindowContext, world: &mut World) -> bool {
        if self.width == 0 || self.width > WorldRegion::MAX_WIDTH {
            wc.open_message_box("Invalid Width", format!("The region with must be between 1 and {}", WorldRegion::MAX_WIDTH));
            return false;
        }
        if self.height == 0 || self.height > WorldRegion::MAX_HEIGHT {
            wc.open_message_box("Invalid Height", format!("The region height must be between 1 and {}", WorldRegion::MAX_HEIGHT));
            return false;
        }

        if let Some(region) = world.regions.get_mut(self.region_index) {
            region.name.clear();
            region.name.push_str(&self.name);
            region.x = self.x;
            region.y = self.y;
            region.width = self.width;
            region.height = self.height;
        }
        true
    }

    pub fn show(&mut self, wc: &mut WindowContext, world: &mut World) {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 300.0, "World Region Properties", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_region_prop_grid", world.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("X:");
                        ui.add(egui::Slider::new(&mut self.x, 0..=255));
                        ui.end_row();

                        ui.label("Y:");
                        ui.add(egui::Slider::new(&mut self.y, 0..=255));
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1..=WorldRegion::MAX_WIDTH));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1..=WorldRegion::MAX_HEIGHT));
                        ui.end_row();

                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() && self.confirm(wc, world) {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
    }
}
