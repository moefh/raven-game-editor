use super::CustomColors;
use super::super::{
    AssetEditorBase,
    WindowContext,
};

pub struct CustomColorsDialog {
    pub open: bool,
    pub dlg_id: egui::Id,
    pub dlg_grid_id: egui::Id,
    pub changed: bool,
}

impl CustomColorsDialog {
    pub fn new() -> Self {
        CustomColorsDialog {
            open: false,
            dlg_id: egui::Id::new("dlg_map_custom_colors"),
            dlg_grid_id: egui::Id::new("dlg_map_custom_colors_grid"),
            changed: false,
        }
    }

    pub fn set_open(
        &mut self,
        wc: &mut WindowContext,
    ) {
        self.changed = false;
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn show_custom_color_toggle(&mut self, ui: &mut egui::Ui, use_custom_color: &mut bool, custom_color: &mut egui::Color32) {
        ui.horizontal(|ui| {
            let btn_label = if *use_custom_color { "Use custom:" } else { "Use default" };
            if ui.add(
                egui::Button::new(btn_label).selected(*use_custom_color)
            ).on_hover_text("Toggle between custom/default color").clicked() {
                *use_custom_color = ! *use_custom_color;
                self.changed = true;
            }
            ui.add_space(2.0);
            if *use_custom_color {
                let orig_rgba = (*custom_color).into();
                let mut rgba = orig_rgba;
                egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                if orig_rgba != rgba {
                    *custom_color = rgba.into();
                    self.changed = true;
                }
            }
        });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        custom_colors: &mut CustomColors,
    ) -> bool {
        if ! self.open { return false; }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, 350.0, "Custom Colors", |ui, _wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(self.dlg_grid_id)
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Grid color:");
                        self.show_custom_color_toggle(ui, &mut custom_colors.use_grid_color, &mut custom_colors.grid_color);
                        ui.end_row();

                        ui.label("BG color:");
                        self.show_custom_color_toggle(ui, &mut custom_colors.use_bg_color, &mut custom_colors.bg_color);
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Ok").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.close(wc);
        }
        self.changed
    }
}
