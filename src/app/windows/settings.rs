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

pub fn show_editor_settings(window: &mut super::AppWindow, wc: &mut super::super::WindowContext) {
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
        .id(window.id)
        .open(&mut window.open)
        .enabled(! wc.sys_dialogs.has_open_dialog())
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
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
                    });
            });

            egui::CollapsingHeader::new("Editor Colors").default_open(true).show(ui, |ui| {
                egui::Grid::new("editor_settings_grid_colors")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        color_setting(ui, "Image BG:", &mut [&mut wc.settings.image_bg_color]);
                        color_setting(ui, "Color Picker BG:", &mut [&mut wc.settings.color_picker_bg_color]);
                        color_setting(ui, "Image Grid:", &mut [&mut wc.settings.image_grid_color]);
                        color_setting(ui, "Map Grid:", &mut [&mut wc.settings.map_grid_color]);
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

                        color_setting(ui, "Colors:", &mut [&mut wc.settings.marching_ants_color1, &mut wc.settings.marching_ants_color2]);
                    });
            });

            ui.add_space(20.0);
            if ui.button("Save").clicked() {
                wc.settings.save(wc.logger);
            }
        });
}
