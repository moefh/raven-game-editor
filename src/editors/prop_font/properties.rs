use crate::app::WindowContext;
use crate::image::ImageCollection;
use crate::data_asset::PropFont;

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub height: f32,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            height: 0.0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_prop_font_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, prop_font: &PropFont) {
        self.name.clear();
        self.name.push_str(&prop_font.asset.name);
        self.height = prop_font.height as f32;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, prop_font: &mut PropFont) {
        prop_font.asset.name.clear();
        prop_font.asset.name.push_str(&self.name);

        let height = self.height as u32;
        if height != prop_font.height {
            prop_font.resize(height * 2, height, PropFont::NUM_CHARS, 0x0c);
            prop_font.max_width = 2 * height;
            prop_font.height = height;
            for char_width in prop_font.char_widths.iter_mut() {
                if *char_width > (2 * height) as u8 {
                    *char_width = (2 * height) as u8;
                }
            }
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, prop_font: &mut PropFont) -> bool {
        if egui::Modal::new(Self::id()).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Prop Font Properties");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(format!("editor_panel_{}_prop_grid", prop_font.asset.id))
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
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
                        self.confirm(prop_font);
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
