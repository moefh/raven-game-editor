pub fn show_editor_status(window: &mut super::AppWindow, wc: &super::super::WindowContext) {
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
    egui::Window::new("Editor Status")
        .id(window.id)
        .open(&mut window.open)
        .enabled(! wc.sys_dialogs.has_open_dialog())
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .show(wc.egui.ctx, |ui| {
            //egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                wc.egui.ctx.texture_ui(ui);
            //});
        });
}
