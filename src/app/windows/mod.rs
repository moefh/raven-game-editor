mod settings;
mod status;
mod log_window;
mod properties;
mod check;

use super::WindowContext;

pub use settings::SettingsWindow;
pub use status::StatusWindow;
pub use log_window::LogWindow;
pub use properties::PropertiesWindow;
pub use check::CheckWindow;

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
        let x = wc.window_space.min.x + 10.0;
        let y = wc.window_space.min.y + 10.0;
        egui::Rect {
            min: egui::Pos2::new(x, y),
            max: egui::Pos2::new(x + width, y + height),
        }
    }

    pub fn create_window<'a>(&'a mut self, wc: &WindowContext, title: &str, default_rect: egui::Rect) -> egui::Window<'a> {
        egui::Window::new(title)
            .id(self.id)
            .open(&mut self.open)
            .enabled(! wc.sys_dialogs.has_open_dialog())
            .default_rect(default_rect)
            .max_width(wc.window_space.width())
            .max_height(wc.window_space.height())
            .constrain_to(wc.window_space)
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

    pub fn show_properties(&mut self, wc: &WindowContext, vga_sync_bits: &mut u8, project_prefix: &mut String) {
        self.properties.show(wc, vga_sync_bits, project_prefix);
    }

    pub fn show_settings(&mut self, wc: &mut WindowContext) {
        self.settings.show(wc);
    }

    pub fn show_status(&mut self, wc: &WindowContext) {
        self.status.show(wc);
    }

    pub fn show_log_window(&mut self, wc: &WindowContext) {
        self.log_window.show(wc);
    }

    pub fn show_check(&mut self, wc: &WindowContext) {
        self.check.show(wc);
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
