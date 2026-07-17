use crate::misc::IMAGES;

use super::{
    AppWindowBase,
    AppWindowAction,
};
use super::super::WindowContext;

pub struct LogWindow {
    pub base: AppWindowBase,
}

impl LogWindow {
    pub fn new(base: AppWindowBase) -> Self {
        LogWindow {
            base,
        }
    }

    pub fn toggle_open(&mut self) {
        self.base.open = ! self.base.open;
    }

    pub fn show(&mut self, wc: &mut WindowContext) -> AppWindowAction {
        let default_rect = self.base.default_rect(wc, 700.0, 300.0);
        self.base.show_window(wc, default_rect, [300.0, 200.0], |ui, wc, base| {
            let action = base.show_title_bar(ui, Some(IMAGES.log), "Log");
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                        ui.label(wc.logger.read());
                    });
                });
            });
            action
        })
    }
}
