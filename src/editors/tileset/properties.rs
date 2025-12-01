use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::Tileset;

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub num_tiles: f32,
    pub sel_color: u8,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            num_tiles: 0.0,
            sel_color: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, tileset: &Tileset, sel_color: u8) {
        self.name.clear();
        self.name.push_str(&tileset.asset.name);
        self.num_tiles = tileset.num_tiles as f32;
        self.sel_color = sel_color;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, tileset: &mut Tileset) {
        tileset.asset.name.clear();
        tileset.asset.name.push_str(&self.name);
        if self.num_tiles as u32 != tileset.num_tiles {
            let image = ImageCollection::from_asset(tileset);
            image.resize(tileset.width, tileset.height, self.num_tiles as u32, &mut tileset.data, self.sel_color);
            tileset.num_tiles = self.num_tiles as u32;
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            ui.set_width(350.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Tileset Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", tileset.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();

                            ui.label("Num tiles:");
                            ui.add(egui::Slider::new(&mut self.num_tiles, 1.0..=255.0).step_by(1.0));
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(tileset);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
