use super::AppWindow;
use super::super::WindowContext;

use crate::data_asset::{DataAssetId, DataAssetStore};
use crate::checker::CheckResult;

pub struct CheckWindow {
    pub base: AppWindow,
    pub result: Option<CheckResult>,
}

impl CheckWindow {
    pub fn new(base: AppWindow) -> Self {
        CheckWindow {
            base,
            result: None,
        }
    }

    pub fn clear(&mut self) {
        self.result = None;
    }

    pub fn run_check(&mut self, store: &DataAssetStore) {
        self.result = Some(CheckResult::check_project(store));
    }

    pub fn show(&mut self, wc: &WindowContext, store: &DataAssetStore) -> Option<DataAssetId> {
        let default_rect = egui::Rect {
            min: egui::Pos2::new(wc.window_space.min.x + 5.0, wc.window_space.max.y - 130.0),
            max: wc.window_space.max - egui::Vec2::splat(20.0),
        };
        let mut open_asset_id = None;
        self.base.create_window(wc, "✔ Project Check", default_rect).show(wc.egui.ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    if let Some(result) = &self.result {
                        ui.label(format!("[{}]", result.timestamp));
                        let num_assets = result.num_assets_checked();
                        let num_assets_with_problems = result.num_assets_with_problems();
                        if num_assets_with_problems > 0 {
                            ui.label("=== PROBLEMS FOUND =========================================");
                            for (asset_id, problems) in &result.asset_problems {
                                if ! problems.is_empty() {
                                    match store.assets.get_asset(*asset_id) {
                                        Some(asset) => {
                                            ui.horizontal(|ui| {
                                                ui.label("->");
                                                if ui.button(&asset.name).clicked() {
                                                    open_asset_id = Some(asset.id);
                                                }
                                            });
                                        }
                                        None => { ui.label("-> <unknown asset>:"); }
                                    }
                                    for problem in problems {
                                        problem.log(ui);
                                    }
                                }
                            }
                            ui.label("============================================================");
                        }
                        ui.label(format!("DONE: {}/{} assets ok", num_assets - num_assets_with_problems, num_assets));
                    } else {
                        ui.label("(Press F5 to check)");
                    }
                });
            });
        });
        open_asset_id
    }
}
