pub fn show_editor_settings(ctx: &egui::Context, window_space: egui::Rect, window_open: &mut bool) {
    let window_id = egui::Id::new("editor_settings");
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
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .open(window_open).show(ctx, |ui| {
            egui::ScrollArea::neither().auto_shrink([false, false]).show(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
}
