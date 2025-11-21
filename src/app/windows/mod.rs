pub mod settings;
pub mod log_window;
pub mod properties;

use super::WindowContext;

pub struct AppWindows {
    pub settings_open: bool,
    pub properties_open: bool,
    pub log_window_open: bool,
}

impl AppWindows {

    pub fn new() -> Self {
        AppWindows {
            settings_open: false,
            properties_open: false,
            log_window_open: false,
        }
    }

    pub fn show_properties(&mut self, wc: &WindowContext, vga_sync_bits: &mut u8, project_prefix: &mut String) {
        properties::show_project_properties(wc, &mut self.properties_open, vga_sync_bits, project_prefix);
    }

    pub fn show_settings(&mut self, wc: &WindowContext) {
        settings::show_editor_settings(wc, &mut self.settings_open);
    }

    pub fn show_log_window(&mut self, wc: &WindowContext) {
        log_window::show_log_window(wc, &mut self.log_window_open);
    }

}
