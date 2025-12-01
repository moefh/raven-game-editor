use std::cell::RefCell;
use std::sync::LazyLock;

use super::AppWindow;
use super::super::WindowContext;

use crate::data_asset::{DataAssetStore, StringLogger};

static TIMESTAMP_FORMAT: LazyLock<Vec<time::format_description::BorrowedFormatItem<'_>>> = LazyLock::new(
    || time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap());

pub struct CheckWindow {
    pub base: AppWindow,
    pub log: RefCell<StringLogger>,
}

impl CheckWindow {
    pub fn new(base: AppWindow) -> Self {
        CheckWindow {
            base,
            log: RefCell::new(StringLogger::new(false)),
        }
    }

    fn clear_log(&self) {
        if let Ok(mut log) = self.log.try_borrow_mut() {
            log.clear();
        }
    }

    fn log(&self, line: impl AsRef<str>) {
        if let Ok(mut log) = self.log.try_borrow_mut() {
            log.log(line);
        }
    }

    pub fn run_check(&mut self, _store: &DataAssetStore) {
        self.clear_log();
        if let Ok(now) = time::OffsetDateTime::now_local() && let Ok(timestamp) = now.format(&TIMESTAMP_FORMAT) {
            self.log(format!("[{}]", timestamp));
        } else {
            self.log("[<unknown time>]");
        }
        self.log("DONE: 0/0 checks passed, 0/0 failed");
    }

    pub fn show(&mut self, wc: &WindowContext) {
        let default_rect = self.base.default_rect(wc, 600.0, 300.0);
        self.base.create_window(wc, "✔ Project Check", default_rect).show(wc.egui.ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    match self.log.try_borrow() {
                        Ok(log) => ui.label(log.read()),
                        Err(e) => ui.label(format!("ERROR: can't read check result:\n{}\n", e)),
                    }
                });
            });
        });
    }
}
