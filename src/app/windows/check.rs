use super::AppWindow;
use super::super::WindowContext;

use crate::misc::IMAGES;
use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};
use crate::checker::CheckResult;

enum WindowAction {
    Close,
    None,
}

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
            if wc.is_window_on_top(self.base.id) {
                wc.egui.ctx.global_style().visuals.widgets.open.weak_bg_fill
            } else {
                wc.egui.ctx.global_style().visuals.faint_bg_color
            }
        }
    }

    fn show_title(ui: &mut egui::Ui, has_problems: bool) -> WindowAction {
        let frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 0 });
        let action = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                //TODO: ui.add(egui::Image::new(include_ref_image!(title_image)).max_size(egui::Vec2::splat(16.0)).shrink_to_fit());
                let label = if has_problems {
                    egui::Label::new("\u{26a0} Project Check")
                } else {
                    egui::Label::new("\u{2714} Project Check")
                };
                ui.add(label.selectable(false));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);
                    if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                        WindowAction::Close
                    } else {
                        WindowAction::None
                    }
                }).inner
            }).inner
        }).inner;

        let size = egui::Vec2::new(ui.available_size_before_wrap().x, 1.0);
        let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
        ui.painter().hline(
            rect.left()..=rect.right(),
            rect.bottom() + 2.0,
            ui.style().visuals.window_stroke
        );

        action
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
        let frame = egui::Frame::window(&wc.egui.ctx.global_style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
            .fill(self.title_bg_color(wc, self.result.as_ref().map(|r| r.num_assets_with_problems() != 0).unwrap_or(false)));
        let (action, open_asset_id) = self.base.create_window(wc, "Project Check", default_rect)
            .frame(frame)
            .title_bar(false)
            .min_size(min_size)
            .show(wc.egui.ctx, |ui| {
                if let Some(result) = &self.result {
                    let action = Self::show_title(ui, result.num_assets_with_problems() != 0);
                    let open_asset_id = Self::show_result(ui, wc, result, store);
                    (action, open_asset_id)
                } else {
                    let action = Self::show_title(ui, false);
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("(Press F5 to check)");
                    });
                    (action, None)
                }
            }).map(|r| r.inner)??;
        if matches!(action, WindowAction::Close) {
            self.base.open = false;
        }
        open_asset_id
    }
}
