pub fn show_log_window(window: &mut super::AppWindow, wc: &super::super::WindowContext) {
    let window_space = wc.window_space;
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
        .id(window.id)
        .open(&mut window.open)
        .enabled(! wc.sys_dialogs.has_open_dialog())
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .show(wc.egui.ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    ui.label(wc.logger.read());
                });
            });
        });
}
