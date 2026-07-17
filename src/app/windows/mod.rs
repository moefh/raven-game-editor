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

pub enum AppWindowResize {
    FixedSize,
    MinSize(egui::Vec2),
}

impl From<[f32; 2]> for AppWindowResize {
    fn from(val: [f32; 2]) -> Self { AppWindowResize::MinSize(val.into()) }
}

impl From<[f32; 0]> for AppWindowResize {
    fn from(_val: [f32; 0]) -> AppWindowResize { AppWindowResize::FixedSize }
}

pub enum AppWindowAction {
    None,
    CloseWindow(egui::Id),
    ActivateAssetEditor(DataAssetId),
}

impl AppWindowAction {
    pub fn is_some(&self) -> bool {
        ! matches!(self, AppWindowAction::None)
    }
}

pub struct AppWindowBase {
    pub id: egui::Id,
    pub open: bool,
}

impl AppWindowBase {
    pub fn new(id_str: &str) -> Self {
        AppWindowBase {
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

    pub fn show_title_bar(&self, ui: &mut egui::Ui, image: Option<egui::ImageSource>, title: &str) -> AppWindowAction {
        let frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 0 });
        let action = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                if let Some(image) = image {
                    ui.add(egui::Image::new(image).max_size(egui::Vec2::splat(16.0)).shrink_to_fit());
                }
                ui.add(egui::Label::new(title).selectable(false));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);
                    if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                        AppWindowAction::CloseWindow(self.id)
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

    pub fn build_window_frame(&self, wc: &WindowContext) -> egui::Frame {
        egui::Frame::window(&wc.egui.ctx.global_style())
            .outer_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
            .inner_margin(egui::Margin { left: 0, right: 0, top: 2, bottom: 0 })
            .fill(self.title_bg_color(wc))
    }

    pub fn show_window(
        &mut self,
        wc: &mut WindowContext,
        default_rect: egui::Rect,
        resize: impl Into<AppWindowResize>,
        show_fn: impl FnOnce(&mut egui::Ui, &mut WindowContext, &mut AppWindowBase) -> AppWindowAction
    ) -> AppWindowAction {
        let mut open = self.open;
        let window = egui::Window::new("")
            .id(self.id)
            .frame(self.build_window_frame(wc))
            .open(&mut open)
            .title_bar(false)
            .enabled(! wc.sys_dialogs.has_open_dialog())
            .default_rect(default_rect)
            .max_size(wc.window_space.size())
            .constrain_to(wc.window_space);
        let window = match resize.into() {
            AppWindowResize::FixedSize => { window.resizable(false) }
            AppWindowResize::MinSize(min_size) => { window.min_size(min_size) }
        };
        let action = window.show(wc.egui.ctx, |ui| {
            show_fn(ui, wc, self)
        }).map(|r| r.inner).unwrap_or(Some(AppWindowAction::None)).unwrap_or(AppWindowAction::None);
        self.open = open;
        action
    }
}

pub struct AppWindowsCollection {
    pub settings: SettingsWindow,
    pub status: StatusWindow,
    pub properties: PropertiesWindow,
    pub log_window: LogWindow,
    pub check: CheckWindow,
}

impl AppWindowsCollection {
    fn new() -> Self {
        AppWindowsCollection {
            settings: SettingsWindow::new(AppWindowBase::new("app_settings")),
            status: StatusWindow::new(AppWindowBase::new("project_status")),
            properties: PropertiesWindow::new(AppWindowBase::new("project_properties")),
            log_window: LogWindow::new(AppWindowBase::new("project_log_window")),
            check: CheckWindow::new(AppWindowBase::new("check_window")),
        }
    }

    fn get_ids(&mut self, window_ids: &mut Vec<egui::Id>) {
        window_ids.push(self.settings.base.id);
        window_ids.push(self.properties.base.id);
        window_ids.push(self.status.base.id);
        window_ids.push(self.log_window.base.id);
        window_ids.push(self.check.base.id);
    }

    fn get_base_window(&self, window_id: egui::Id) -> Option<&AppWindowBase> {
        if window_id == self.settings.base.id { return Some(&self.settings.base) }
        if window_id == self.properties.base.id { return Some(&self.properties.base) }
        if window_id == self.status.base.id { return Some(&self.status.base) }
        if window_id == self.log_window.base.id { return Some(&self.log_window.base) }
        if window_id == self.check.base.id { return Some(&self.check.base) }
        None
    }

    fn get_base_window_mut(&mut self, window_id: egui::Id) -> Option<&mut AppWindowBase> {
        if window_id == self.settings.base.id { return Some(&mut self.settings.base) }
        if window_id == self.properties.base.id { return Some(&mut self.properties.base) }
        if window_id == self.status.base.id { return Some(&mut self.status.base) }
        if window_id == self.log_window.base.id { return Some(&mut self.log_window.base) }
        if window_id == self.check.base.id { return Some(&mut self.check.base) }
        None
    }

    fn add_window_action(actions: &mut Vec<AppWindowAction>, action: AppWindowAction) {
        if action.is_some() {
            actions.push(action)
        }
    }

    fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> Vec<AppWindowAction> {
        let mut actions = Vec::new();
        Self::add_window_action(&mut actions, self.settings.show(wc));
        Self::add_window_action(&mut actions, self.properties.show(wc, store));
        Self::add_window_action(&mut actions, self.status.show(wc, store));
        Self::add_window_action(&mut actions, self.log_window.show(wc));
        Self::add_window_action(&mut actions, self.check.show(wc, store));
        actions
    }
}

pub struct AppWindows {
    pub window_ids: Vec<egui::Id>,
    pub collection: AppWindowsCollection,
    pub some_closed_last_frame: bool,
}

impl AppWindows {
    pub fn new() -> Self {
        AppWindows {
            window_ids: Vec::new(),
            collection: AppWindowsCollection::new(),
            some_closed_last_frame: false,
        }
    }

    pub fn get_ids(&mut self) -> &[egui::Id] {
        if self.window_ids.is_empty() {
            self.collection.get_ids(&mut self.window_ids);
        }
        &self.window_ids
    }

    pub fn get_open_ids(&mut self) -> impl Iterator<Item = egui::Id> {
        self.get_ids();
        self.window_ids.iter().copied().filter(|id| {
            self.collection.get_base_window(*id).map(|w| w.open).unwrap_or(false)
        })
    }

    pub fn close(&mut self, window_id: egui::Id) -> bool {
        if let Some(base) = self.collection.get_base_window_mut(window_id) {
            self.some_closed_last_frame = true;
            base.open = false;
            true
        } else {
            false
        }
    }

    pub fn clear_project(&mut self) {
        self.collection.check.clear();
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) -> Vec<AppWindowAction> {
        self.some_closed_last_frame = false;
        self.collection.show(wc, store)
    }

    pub fn open_log_window(&mut self) { self.collection.log_window.base.open = true; }
    pub fn open_properties(&mut self) { self.collection.properties.base.open = true; }
    pub fn open_settings(&mut self) { self.collection.settings.base.open = true; }
    pub fn open_status(&mut self) { self.collection.status.base.open = true; }
    pub fn open_check(&mut self) { self.collection.check.base.open = true; }

    pub fn run_check(&mut self, store: &DataAssetStore) {
        self.collection.check.run_check(store);
    }
}
