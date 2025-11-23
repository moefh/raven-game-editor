fn color_setting(ui: &mut egui::Ui, label: &'static str, color: &mut egui::Color32) {
    ui.label(label);
    let mut rgba = (*color).into();
    egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
    *color = rgba.into();
    ui.end_row();
}

pub fn show_editor_settings(wc: &mut super::super::WindowContext, window_open: &mut bool) {
    let window_id = egui::Id::new("editor_settings");
    let window_space = wc.window_space;
    let default_rect = egui::Rect {
        min: egui::Pos2 {
            x : window_space.min.x + 10.0,
            y : window_space.min.y + 10.0,
        },
        max: egui::Pos2 {
            x: 600.0,
            y: 300.0,
        }
    };
    egui::Window::new("Editor Settings")
        .id(window_id)
        .enabled(! wc.sys_dialogs.has_open_dialog())
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .open(window_open).show(wc.egui.ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                egui::Grid::new("editor_settings_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Theme:");
                        egui::widgets::global_theme_preference_buttons(ui);
                        match ui.ctx().theme() {
                            egui::Theme::Light => if wc.settings.theme != "light" { wc.settings.theme = "light".to_owned(); }
                            egui::Theme::Dark => if wc.settings.theme != "dark" { wc.settings.theme = "dark".to_owned(); }
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

                        color_setting(ui, "Image grid:", &mut wc.settings.image_grid_color);
                        color_setting(ui, "Map grid:", &mut wc.settings.map_grid_color);
                    });
            });
            ui.add_space(10.0);
            if ui.button("Save").clicked() {
                wc.settings.save(wc.logger);
            }
        });
}
