use super::AppWindow;
use super::super::WindowContext;

pub struct StatusWindow {
    pub base: AppWindow,
}

impl StatusWindow {
    pub fn new(base: AppWindow) -> Self {
        StatusWindow {
            base,
        }
    }

    pub fn show(&mut self, wc: &WindowContext) {
        let default_rect = self.base.default_rect(wc, 400.0, 300.0);
        self.base.create_window(wc, "Editor Status", default_rect).show(wc.egui.ctx, |ui| {
            wc.egui.ctx.texture_ui(ui);
        });
    }
}
