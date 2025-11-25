use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::Font;

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub width: f32,
    pub height: f32,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn set_open(&mut self, font: &Font) {
        self.name.clear();
        self.name.push_str(&font.asset.name);
        self.width = font.width as f32;
        self.height = font.height as f32;
        self.open = true;
    }

    fn confirm(&mut self, font: &mut Font) {
        font.asset.name.clear();
        font.asset.name.push_str(&self.name);

        let width = self.width as u32;
        let height = self.height as u32;
        if width != font.width || height != font.height {
            let image = ImageCollection::from_asset(font);
            image.resize(width, height, Font::NUM_CHARS, &mut font.data, 0x0c);
            font.width = width;
            font.height = height;
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &WindowContext, font: &mut Font) -> bool {
        if egui::Modal::new(egui::Id::new("dlg_font_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Font Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", font.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();

                            ui.label("Width:");
                            ui.add(egui::Slider::new(&mut self.width, 4.0..=48.0).step_by(1.0));
                            ui.end_row();

                            ui.label("Height:");
                            ui.add(egui::Slider::new(&mut self.height, 4.0..=48.0).step_by(1.0));
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(font);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
