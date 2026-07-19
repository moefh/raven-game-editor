use super::{
    SysDialogs,
    AppWindowTracker,
    AppSettings,
    create_dialog_window,
};

use egui::{Vec2, Rect};

use crate::image::colors::color_to_rgb;
use super::super::editors::ColorPickerPopupWidget;

pub struct ColorsetEditorDialog {
    id: egui::Id,
    open: bool,
    colorset: usize,
    pick_color_index: Option<usize>,
    color_picker_popup: ColorPickerPopupWidget,
}

impl ColorsetEditorDialog {
    const MIN_PICKER_WIDTH: f32 = 150.0;
    const MIN_WINDOW_WIDTH: f32 = Self::MIN_PICKER_WIDTH + 12.0;
    const COLORS_PER_ROW: usize = 4;

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
        wt.set_dialog_open(self.id, self.open);
    }

    fn show_colorset_editor(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            if let Some(colorset) = settings.colorsets.get_custom_colorset_mut(self.colorset) {
                ui.text_edit_singleline(&mut colorset.name);
            }
        });

        let min_size = Vec2::new(Self::MIN_PICKER_WIDTH, Self::MIN_PICKER_WIDTH * 1.2);
        let (response, painter) = ui.allocate_painter(min_size, egui::Sense::click());
        let mut rect = response.rect.shrink(3.0);

        if let Some(colors) = settings.colorsets.get_colorset_colors(self.colorset) {
            let (num_x, num_y) = (Self::COLORS_PER_ROW, colors.len().div_ceil(Self::COLORS_PER_ROW));
            let color_block_size = Vec2::new(rect.width() / (num_x as f32), rect.height() / (num_y as f32));
            rect.max.y = rect.min.y + color_block_size.y * colors.len().div_ceil(Self::COLORS_PER_ROW) as f32;
            for (i, &color) in colors.iter().enumerate() {
                let x = i % Self::COLORS_PER_ROW;
                let y = i / Self::COLORS_PER_ROW;
                let pos = Vec2::new(x as f32, y as f32);
                let item_rect = Rect::from_min_size(rect.min + color_block_size * pos, color_block_size);
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(color));
            }
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::StrokeKind::Outside);

            if response.clicked() && let Some(pos) = response.interact_pointer_pos() {
                let x = ((pos.x - response.rect.min.x) / color_block_size.x).floor().clamp(0.0, 3.0) as usize;
                let y = ((pos.y - response.rect.min.y) / color_block_size.y).floor().max(0.0) as usize;
                let color_index = y*Self::COLORS_PER_ROW + x;
                if colors.get(color_index).is_some() {
                    self.pick_color_index = Some(color_index);
                }
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

        if create_dialog_window(sys_dialogs, ui, self.id, Self::MIN_WINDOW_WIDTH, "Colorset", |ui| {
            self.show_colorset_editor(ui, settings);
            ui.add_space(5.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Close").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wt.set_dialog_open(self.id, self.open);
        }
    }
}
