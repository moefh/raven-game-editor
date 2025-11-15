pub mod settings;
pub mod log_window;
pub mod properties;

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

    pub fn show_properties(&mut self, ctx: &egui::Context, window_space: egui::Rect, vga_sync_bits: &mut u8, project_prefix: &mut String) {
        properties::show_project_properties(ctx, window_space, &mut self.properties_open, vga_sync_bits, project_prefix);
    }

    pub fn show_settings(&mut self, ctx: &egui::Context, window_space: egui::Rect) {
        settings::show_editor_settings(ctx, window_space, &mut self.settings_open);
    }

    pub fn show_log_window(&mut self, ctx: &egui::Context, window_space: egui::Rect, log_text: &String) {
        log_window::show_log_window(ctx, window_space, &mut self.log_window_open, log_text);
    }

}
