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

    pub fn show(&mut self, wc: &WindowContext) {
        let default_rect = self.base.default_rect(wc, 600.0, 300.0);
        self.base.create_window(wc, "Log", default_rect).show(wc.egui.ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    ui.label(wc.logger.read());
                });
            });
        });
    }
}
