mod settings;
mod status;
mod log_window;
mod properties;
mod check;

use crate::misc::IMAGES;
use crate::data_asset::{
    DataAssetId,
    DataAssetStore,
};

use super::WindowContext;

pub use settings::SettingsWindow;
pub use status::StatusWindow;
pub use log_window::LogWindow;
pub use properties::PropertiesWindow;
pub use check::CheckWindow;

pub enum AppWindowAction {
    Close,
    None,
}

pub struct AppWindow {
    pub id: egui::Id,
    pub open: bool,
}

impl AppWindow {
    pub fn new(id_str: &str) -> Self {
        AppWindow {
            id: egui::Id::new(id_str),
            open: false,
        }
    }

    pub fn default_rect(&self, wc: &WindowContext, width: f32, height: f32) -> egui::Rect {
        egui::Rect::from_min_size(wc.window_space.min + egui::Vec2::splat(10.0), egui::Vec2::new(width, height))
    }

    pub fn title_bg_color(&self, wc: &WindowContext) -> egui::Color32 {
        if wc.is_window_on_top(self.id) {
            wc.egui.ctx.global_style().visuals.widgets.open.weak_bg_fill
        } else {
            wc.egui.ctx.global_style().visuals.faint_bg_color
        }
    }

    pub fn show_title_bar(ui: &mut egui::Ui, title: &str) -> AppWindowAction {
        let frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 0 });
        let action = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                ui.add(egui::Label::new(title).selectable(false));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);
                    if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                        AppWindowAction::Close
                    } else {
                        AppWindowAction::None
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

    pub fn build_window_frame(wc: &WindowContext) -> egui::Frame {
        egui::Frame::window(&wc.egui.ctx.global_style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
    }

    pub fn create_window<'a>(&'a mut self, wc: &WindowContext, title: &str, default_rect: egui::Rect) -> egui::Window<'a> {
        let frame = Self::build_window_frame(wc).fill(self.title_bg_color(wc));
        egui::Window::new(title)
            .id(self.id)
            .open(&mut self.open)
            .title_bar(false)
            .frame(frame)
            .enabled(! wc.sys_dialogs.has_open_dialog())
            .default_rect(default_rect)
            .max_size(wc.window_space.size())
            .constrain_to(wc.window_space)
    }

    pub fn run_window_action(&mut self, resp: Option<egui::InnerResponse<Option<AppWindowAction>>>) {
        if let Some(Some(action)) = resp.map(|r| r.inner) && matches!(action, AppWindowAction::Close) {
            self.open = false;
        }
    }
}

pub struct AppWindows {
    pub window_ids: Vec<egui::Id>,
    pub settings: SettingsWindow,
    pub status: StatusWindow,
    pub properties: PropertiesWindow,
    pub log_window: LogWindow,
    pub check: CheckWindow,
}

impl AppWindows {
    pub fn new() -> Self {
        AppWindows {
            window_ids: Vec::new(),
            settings: SettingsWindow::new(AppWindow::new("app_settings")),
            status: StatusWindow::new(AppWindow::new("project_status")),
            properties: PropertiesWindow::new(AppWindow::new("project_properties")),
            log_window: LogWindow::new(AppWindow::new("project_log_window")),
            check: CheckWindow::new(AppWindow::new("check_window")),
        }
    }

    pub fn get_ids(&mut self) -> &[egui::Id] {
        if self.window_ids.is_empty() {
            self.window_ids.push(self.settings.base.id);
            self.window_ids.push(self.properties.base.id);
            self.window_ids.push(self.status.base.id);
            self.window_ids.push(self.log_window.base.id);
            self.window_ids.push(self.check.base.id);
        }
        &self.window_ids
    }

    pub fn clear_project(&mut self) {
        self.check.clear();
    }

    pub fn show_properties(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) {
        self.properties.show(wc, store);
    }

    pub fn show_settings(&mut self, wc: &mut WindowContext) {
        self.settings.show(wc);
    }

    pub fn show_status(&mut self, wc: &WindowContext, store: &DataAssetStore) {
        self.status.show(wc, store);
    }

    pub fn show_log_window(&mut self, wc: &WindowContext) {
        self.log_window.show(wc);
    }

    pub fn show_check(&mut self, wc: &WindowContext, store: &DataAssetStore) -> Option<DataAssetId> {
        self.check.show(wc, store)
    }

    pub fn open_log_window(&mut self) {
        self.log_window.base.open = true;
    }

    pub fn open_properties(&mut self) {
        self.properties.base.open = true;
    }

    pub fn open_settings(&mut self) {
        self.settings.base.open = true;
    }

    pub fn open_status(&mut self) {
        self.status.base.open = true;
    }

    pub fn open_check(&mut self) {
        self.check.base.open = true;
    }
}
