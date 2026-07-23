use crate::image::ImageCollection;
use crate::data_asset::Font;

use super::super::{
    AssetEditorBase,
    WindowContext,
};

pub struct PropertiesDialog {
    pub image_changed: bool,
    pub open: bool,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

impl PropertiesDialog {
    pub fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_font_properties")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, font: &Font) {
        self.name.clear();
        self.name.push_str(&font.asset.name);
        self.width = font.width;
        self.height = font.height;
        self.open = true;
        wc.set_dialog_open(Self::id(), self.open);
    }

    fn confirm(&mut self, font: &mut Font) {
        font.asset.name.clear();
        font.asset.name.push_str(&self.name);

        if self.width != font.width || self.height != font.height {
            font.resize(self.width, self.height, Font::NUM_CHARS, Font::BG_COLOR);
            self.image_changed = true;
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) -> bool {
        if AssetEditorBase::show_dialog_window(wc, Self::id(), 300.0, "Font Properties", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_prop_grid", font.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 4..=48).step_by(1.0));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 4..=48).step_by(1.0));
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
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(Self::id(), self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
