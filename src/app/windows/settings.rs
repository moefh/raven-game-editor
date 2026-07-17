use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::WindowContext;

use crate::image::ColorSet;
use crate::misc::IMAGES;

pub struct SettingsWindow {
    pub base: AppWindowBase,
}

impl SettingsWindow {
    pub fn new(base: AppWindowBase) -> Self {
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

    fn show_main_settings(ui: &mut egui::Ui, wc: &mut WindowContext) {
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
                    ui.horizontal(|ui| {
                        if ui.button("\u{2796}").on_hover_text("Zoom out").clicked() {
                            let zoom = ui.ctx().zoom_factor();
                            if zoom > 0.5 {
                                ui.ctx().set_zoom_factor(((zoom - 0.1) * 10.0).round() / 10.0);
                            }
                        }
                        ui.menu_button(format!("{:3.1}x", wc.egui.ctx.zoom_factor()), |ui| {
                            egui::gui_zoom::zoom_menu_buttons(ui);
                        });
                        if ui.button("\u{2795}").on_hover_text("Zoom in").clicked() {
                            let zoom = ui.ctx().zoom_factor();
                            if zoom < 5.0 {
                                ui.ctx().set_zoom_factor(((zoom + 0.1) * 10.0).round() / 10.0);
                            }
                        }
                    });
                    if wc.settings.zoom != (ui.ctx().zoom_factor() * 100.0).round() as u32 {
                        wc.settings.zoom = (ui.ctx().zoom_factor() * 100.0).round() as u32;
                    }
                    ui.end_row();

                    ui.label("Start maximized:");
                    ui.checkbox(&mut wc.settings.start_maximized, "");
                    ui.end_row();
                });
        });
    }

    fn show_editor_color_settings(ui: &mut egui::Ui, wc: &mut WindowContext) {
        egui::CollapsingHeader::new("Editor Colors").default_open(true).show(ui, |ui| {
            egui::Grid::new("editor_settings_grid_colors")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    Self::color_setting(ui, "Image BG:", &mut [&mut wc.settings.image_bg_color]);
                    Self::color_setting(ui, "Map BG:", &mut [&mut wc.settings.map_bg_color]);
                    Self::color_setting(ui, "Color Picker BG:", &mut [&mut wc.settings.color_picker_bg_color]);
                    Self::color_setting(ui, "Image Grid:", &mut [&mut wc.settings.image_grid_color]);
                    Self::color_setting(ui, "Map Grid:", &mut [&mut wc.settings.map_grid_color]);
                });
        });
    }

    fn show_colorset_settings(ui: &mut egui::Ui, wc: &mut WindowContext) {
        egui::CollapsingHeader::new("Colorsets").default_open(true).show(ui, |ui| {
            if wc.settings.colorsets.get_num_custom_colorsets() > 0 {
                egui::Grid::new("editor_settings_colorsets")
                    .num_columns(3)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for index in wc.settings.colorsets.get_custom_colorset_range() {
                            if let Some((edit, remove)) = match wc.settings.colorsets.get_custom_colorset(index) {
                                Some(colorset) => {
                                    ui.label(&colorset.name);
                                    let edit = ui.add(egui::Button::new("Edit Colors")).clicked();
                                    let remove = ui.add(
                                        egui::Button::image(IMAGES.trash)
                                    ).on_hover_text("Remove colorset").clicked();
                                    ui.end_row();
                                    Some((edit, remove))
                                }
                                None => { None }
                            } {
                                if remove {
                                    wc.settings.colorsets.remove_custom_colorset(index);
                                }
                                if edit {
                                    wc.open_colorset_dialog(index);
                                }
                            }
                        }
                    });

                ui.add_space(5.0);
            }

            if ui.add(egui::Button::new("Add Colorset")).clicked() {
                wc.settings.colorsets.add_custom_colorset(ColorSet::new("new_colorset".to_owned(), vec![]));
            }
        });
    }

    fn show_marching_ants_settings(ui: &mut egui::Ui, wc: &mut WindowContext) {
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

                    Self::color_setting(ui, "Colors:", &mut [&mut wc.settings.marching_ants_color1, &mut wc.settings.marching_ants_color2]);
                });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext) -> AppWindowAction {
        let default_rect = self.base.default_rect(wc, 400.0, 300.0);
        self.base.show_window(wc, default_rect, [400.0, 200.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.properties), "Editor Settings");
            egui::Panel::bottom("editor_settings_bottom").show(ui, |ui| {
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui.button("Save Settings").clicked() {
                            wc.settings.save(wc.logger);
                        }
                    });
                });
                ui.add_space(5.0);
            });
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    Self::show_main_settings(ui, wc);
                    ui.add_space(5.0);
                    Self::show_editor_color_settings(ui, wc);
                    ui.add_space(5.0);
                    Self::show_colorset_settings(ui, wc);
                    ui.add_space(5.0);
                    Self::show_marching_ants_settings(ui, wc);
                });
            });
            action
        })
    }
}
