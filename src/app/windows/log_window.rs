pub fn show_log_window(ctx: &egui::Context, window_space: egui::Rect, window_open: &mut bool, log_text: &String) {
    let window_id = egui::Id::new("log_window");
    let default_rect = egui::Rect {
        min: egui::Pos2 {
            x : window_space.min.x + 10.0,
            y : window_space.min.y + 10.0,
        },
        max: egui::Pos2 {
            x: 900.0,
            y: 300.0,
        }
    };
    egui::Window::new("Log")
        .id(window_id)
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .open(window_open).show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true), |ui| {
                    ui.label(log_text);
                });
            });
        });
}
