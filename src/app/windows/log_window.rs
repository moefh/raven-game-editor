use super::AppWindow;
use super::super::WindowContext;

pub struct LogWindow {
    pub base: AppWindow,
}

impl LogWindow {
    pub fn new(base: AppWindow) -> Self {
        LogWindow {
            base,
        }
    }

    pub fn toggle_open(&mut self) {
        self.base.open = ! self.base.open;
    }

    pub fn show(&mut self, wc: &WindowContext) {
        let title = "Log";
        let default_rect = self.base.default_rect(wc, 700.0, 300.0);
        let resp = self.base.create_window(wc, title, default_rect).min_size([300.0, 200.0]).show(wc.egui.ctx, |ui| {
            let action = AppWindow::show_title_bar(ui, title);
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                        ui.label(wc.logger.read());
                    });
                });
            });
            action
        });
        self.base.run_window_action(resp);
    }
}
