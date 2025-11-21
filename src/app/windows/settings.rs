pub fn show_editor_settings(wc: &super::super::WindowContext, window_open: &mut bool) {
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
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(format!("Zoom: {}", wc.egui.ctx.zoom_factor()));
                    ui.menu_button("Change", |ui| {
                        egui::gui_zoom::zoom_menu_buttons(ui);
                    });
                });
                ui.add_space(5.0);
                wc.egui.ctx.texture_ui(ui);
            });
        });
}
