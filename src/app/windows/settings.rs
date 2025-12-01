use super::AppWindow;
use super::super::WindowContext;

pub struct SettingsWindow {
    pub base: AppWindow,
}

impl SettingsWindow {
    pub fn new(base: AppWindow) -> Self {
        SettingsWindow {
            base,
        }
    }

    fn color_setting(ui: &mut egui::Ui, label: &'static str, colors: &mut [&mut egui::Color32]) {
        ui.label(label);
        ui.horizontal(|ui| {
            for color in colors {
                let mut rgba = (**color).into();
                egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                **color = rgba.into();
            }
        });
        ui.end_row();
    }

    pub fn show(&mut self, wc: &mut WindowContext) {
        let default_rect = self.base.default_rect(wc, 400.0, 200.0);
        self.base.create_window(wc, "Editor Settings", default_rect)
            .vscroll(true)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(wc.egui.ctx, |ui| {
                egui::CollapsingHeader::new("Main Settings").default_open(true).show(ui, |ui| {
                    egui::Grid::new("editor_settings_main")
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Theme:");
                            egui::widgets::global_theme_preference_buttons(ui);
                            match ui.ctx().options(|opt| { opt.theme_preference }) {
                                egui::ThemePreference::System => if wc.settings.theme != "system" { wc.settings.theme = "system".to_owned(); }
                                egui::ThemePreference::Light => if wc.settings.theme != "light" { wc.settings.theme = "light".to_owned(); }
                                egui::ThemePreference::Dark => if wc.settings.theme != "dark" { wc.settings.theme = "dark".to_owned(); }
                            }
                            ui.end_row();

                            ui.label("Zoom:");
                            ui.menu_button(format!("{:3.1}x", wc.egui.ctx.zoom_factor()), |ui| {
                                egui::gui_zoom::zoom_menu_buttons(ui);
                            });
                            if wc.settings.zoom != (ui.ctx().zoom_factor() * 100.0).round() as u32 {
                                wc.settings.zoom = (ui.ctx().zoom_factor() * 100.0).round() as u32;
                            }
                            ui.end_row();
                        });
                });

                egui::CollapsingHeader::new("Editor Colors").default_open(true).show(ui, |ui| {
                    egui::Grid::new("editor_settings_grid_colors")
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            Self::color_setting(ui, "Image BG:", &mut [&mut wc.settings.image_bg_color]);
                            Self::color_setting(ui, "Color Picker BG:", &mut [&mut wc.settings.color_picker_bg_color]);
                            Self::color_setting(ui, "Image Grid:", &mut [&mut wc.settings.image_grid_color]);
                            Self::color_setting(ui, "Map Grid:", &mut [&mut wc.settings.map_grid_color]);
                        });
                });


                egui::CollapsingHeader::new("Marching Ants").default_open(true).show(ui, |ui| {
                    egui::Grid::new("editor_settings_marching_ants")
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Delay:");
                            ui.add(egui::Slider::new(&mut wc.settings.marching_ants_delay, 50..=500));
                            ui.end_row();

                            ui.label("Dash size:");
                            ui.add(egui::Slider::new(&mut wc.settings.marching_ants_dash_size, 2..=16));
                            ui.end_row();

                            ui.label("Thickness:");
                            ui.add(egui::Slider::new(&mut wc.settings.marching_ants_thickness, 1..=5));
                            ui.end_row();

                            Self::color_setting(ui, "Colors:", &mut [&mut wc.settings.marching_ants_color1,
                                                                     &mut wc.settings.marching_ants_color2]);
                        });
                });

                ui.add_space(20.0);
                if ui.button("Save").clicked() {
                    wc.settings.save(wc.logger);
                }
            });
    }
}
