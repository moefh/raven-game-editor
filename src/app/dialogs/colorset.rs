use super::{
    SysDialogs,
    AppWindowTracker,
    AppSettings,
};

use egui::{Vec2, Pos2, Rect};

use crate::image::colors::color_to_rgb;
use crate::editors::ColorPickerPopupWidget;

pub struct ColorsetEditorDialog {
    id: egui::Id,
    open: bool,
    colorset: usize,
    pick_color_index: Option<usize>,
    color_picker_popup: ColorPickerPopupWidget,
}

impl ColorsetEditorDialog {
    const MIN_WINDOW_WIDTH: f32 = 300.0;
    const MIN_PICKER_WIDTH: f32 = 300.0;

    pub fn new() -> Self {
        ColorsetEditorDialog {
            id: egui::Id::new("dlg_colorset_window"),
            open: false,
            colorset: 0,
            pick_color_index: None,
            color_picker_popup: ColorPickerPopupWidget::new(egui::Id::new("dlg_colorset_picker"), true),
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker, colorset: usize) {
        self.colorset = colorset;
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    fn show_colorset_editor(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            if let Some(colorset) = settings.colorsets.get_custom_colorset_mut(self.colorset) {
                ui.text_edit_singleline(&mut colorset.name);
            }
        });

        let min_size = Vec2::new(Self::MIN_PICKER_WIDTH, Self::MIN_PICKER_WIDTH * 7.0 / 8.0);
        let (response, painter) = ui.allocate_painter(min_size, egui::Sense::click());

        let mut rect = response.rect.shrink(3.0);
        let color_block_w = rect.width() / 4.0;
        let color_block_h = color_block_w / 2.0;

        if let Some(colors) = settings.colorsets.get_colorset_colors(self.colorset) {
            rect.max.y = rect.min.y + color_block_h * colors.len().div_ceil(4) as f32;
            for (i, &color) in colors.iter().enumerate() {
                let x = i % 4;
                let y = i / 4;

                let item_rect = Rect {
                    min: Pos2::new(rect.min.x + (x     as f32) * color_block_w, rect.min.y + (y     as f32) * color_block_h),
                    max: Pos2::new(rect.min.x + ((x+1) as f32) * color_block_w, rect.min.y + ((y+1) as f32) * color_block_h),
                };
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
            }
        }
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::StrokeKind::Outside);

        if response.clicked() && let Some(pos) = response.interact_pointer_pos() {
            let x = ((pos.x - response.rect.min.x) / color_block_w).floor().clamp(0.0, 3.0) as usize;
            let y = ((pos.y - response.rect.min.y) / color_block_h).floor().max(0.0) as usize;
            let color_index = y*4 + x;
            if let Some(colors) = settings.colorsets.get_colorset_colors(self.colorset) &&
                colors.get(color_index).is_some() {
                    self.pick_color_index = Some(color_index);
                }
        }

        if let Some(color_index) = self.pick_color_index {
            let mut color = settings.colorsets.get_colorset_colors(self.colorset)
                .and_then(|colors| colors.get(color_index).copied())
                .unwrap_or(0);
            self.color_picker_popup.show(&response, settings, &mut color);
            if let Some(colorset) = settings.colorsets.get_custom_colorset_mut(self.colorset) {
                colorset.colors[color_index] = color;
            }
            if self.color_picker_popup.close {
                self.pick_color_index = None;
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs, settings: &mut AppSettings) {
        if ! self.open { return; }

        if egui::Modal::new(self.id).show(ui.ctx(), |ui| {
            sys_dialogs.block_ui(ui);

            ui.set_width(Self::MIN_WINDOW_WIDTH);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Colorset");
                ui.separator();
                self.show_colorset_editor(ui, settings);
            });
        }).should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
    }
}
