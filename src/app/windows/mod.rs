pub mod settings;
pub mod status;
pub mod log_window;
pub mod properties;

use super::WindowContext;

pub struct AppWindow {
    pub id: egui::Id,
    pub open: bool,
}

impl AppWindow {
    pub fn new(id: egui::Id) -> Self {
        AppWindow {
            id,
            open: false,
        }
    }
}

pub struct AppWindows {
    pub window_ids: Vec<egui::Id>,
    pub settings: AppWindow,
    pub status: AppWindow,
    pub properties: AppWindow,
    pub log_window: AppWindow,
}

impl AppWindows {
    pub fn new() -> Self {
        let settings_id = egui::Id::new("project_settings");
        let status_id = egui::Id::new("project_status");
        let properties_id = egui::Id::new("project_properties");
        let log_window_id = egui::Id::new("project_log_window");
        let window_ids = vec![
            settings_id,
            status_id,
            properties_id,
            log_window_id,
        ];
        AppWindows {
            window_ids,
            settings: AppWindow::new(settings_id),
            status: AppWindow::new(status_id),
            properties: AppWindow::new(properties_id),
            log_window: AppWindow::new(log_window_id),
        }
    }

    pub fn get_ids(&self) -> &[egui::Id] {
        &self.window_ids
    }

    pub fn show_properties(&mut self, wc: &WindowContext, vga_sync_bits: &mut u8, project_prefix: &mut String) {
        properties::show_project_properties(&mut self.properties, wc, vga_sync_bits, project_prefix);
    }

    pub fn show_settings(&mut self, wc: &mut WindowContext) {
        settings::show_editor_settings(&mut self.settings, wc);
    }

    pub fn show_status(&mut self, wc: &WindowContext) {
        status::show_editor_status(&mut self.status, wc);
    }

    pub fn show_log_window(&mut self, wc: &WindowContext) {
        log_window::show_log_window(&mut self.log_window, wc);
    }

}
