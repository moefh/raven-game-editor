use super::{
    AppWindow,
    AppWindowAction,
};
use super::super::WindowContext;

use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};
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

    fn title_bg_color(&self, wc: &WindowContext, has_problems: bool) -> egui::Color32 {
        if has_problems && wc.egui.ctx.global_style().visuals.dark_mode {
            if wc.is_window_on_top(self.base.id) {
                egui::Color32::from_rgb(128, 16, 16)
            } else {
                egui::Color32::from_rgb(64, 0, 0)
            }
        } else if has_problems {
            if wc.is_window_on_top(self.base.id) {
                egui::Color32::from_rgb(255, 192, 192)
            } else {
                egui::Color32::from_rgb(255, 224, 224)
            }
        } else {
            self.base.title_bg_color(wc)
        }
    }

    fn show_result(ui: &mut egui::Ui, _wc: &WindowContext, result: &CheckResult, store: &DataAssetStore) -> Option<DataAssetId> {
        let mut open_asset_id = None;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(false), |ui| {
                    let num_assets = result.num_assets_checked();
                    let num_assets_with_problems = result.num_assets_with_problems();
                    ui.label(format!("[{}] {}", result.timestamp, if num_assets_with_problems > 0 { "PROBLEMS DETECTED" } else { "OK" }));
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
                                    problem.log(ui, *asset_id, store);
                                }
                            }
                        }
                        ui.label("============================================================");
                    }

                    if result.merged_samples.is_empty() {
                        ui.label(format!("-> data size: {} bytes", result.data_size));
                    } else {
                        ui.label("-> MOD samples will be merged:");
                        for merge in result.merged_samples.iter() {
                            if let Some(merged_mod) = store.assets.mods.get(&merge.merged_mod_id) &&
                                    let Some(data_mod) = store.assets.mods.get(&merge.data_mod_id) {
                                        ui.label(format!(
                                            "   {}.sample{} to {}.sample{} ({} bytes saved)",
                                            merged_mod.asset.name, merge.merged_sample_index+1,
                                            data_mod.asset.name, merge.data_sample_index+1,
                                            merge.saved_size,
                                        ));
                                    } else {
                                        ui.label("   (error fetching data)");
                                    }
                        }
                        ui.label(format!("-> data size: {} bytes ({} bytes saved by sample merging)",
                            result.data_size, result.merged_samples_saved_size));
                    }

                    ui.label(format!("DONE: {}/{} assets ok", num_assets - num_assets_with_problems, num_assets));
                });
            });
        });
        open_asset_id
    }

    pub fn show(&mut self, wc: &WindowContext, store: &DataAssetStore) -> Option<DataAssetId> {
        let default_rect = egui::Rect {
            min: egui::Pos2::new(wc.window_space.min.x + 5.0, wc.window_space.max.y - 150.0),
            max: wc.window_space.max - egui::Vec2::splat(5.0),
        };
        let min_size = egui::Vec2::new(300.0, default_rect.height());
        let frame = AppWindow::build_window_frame(wc).fill(
            self.title_bg_color(wc, self.result.as_ref().map(|r| r.num_assets_with_problems() != 0).unwrap_or(false))
        );
        let (action, open_asset_id) = self.base.create_window(wc, "Project Check", default_rect)
            .frame(frame)
            .min_size(min_size)
            .show(wc.egui.ctx, |ui| {
                if let Some(result) = &self.result {
                    let title = if result.num_assets_with_problems() != 0 {
                        "\u{26a0} Project Check"
                    } else {
                        "\u{2714} Project Check"
                    };
                    let action = AppWindow::show_title_bar(ui, title);
                    let open_asset_id = Self::show_result(ui, wc, result, store);
                    (action, open_asset_id)
                } else {
                    let action = AppWindow::show_title_bar(ui, "Project Check");
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("(Press F5 to check)");
                    });
                    (action, None)
                }
            }).map(|r| r.inner)??;
        if matches!(action, AppWindowAction::Close) {
            self.base.open = false;
        }
        open_asset_id
    }
}
