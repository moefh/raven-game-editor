use super::{
    SysDialogs,
    AppWindowTracker,
    create_dialog_window,
};

pub struct MessageBoxDialog {
    id: egui::Id,
    open: bool,
    title: String,
    text: String,
}

impl MessageBoxDialog {
    const WINDOW_WIDTH: f32 = 350.0;

    pub fn new() -> Self {
        MessageBoxDialog {
            id: egui::Id::new("dlg_message_box"),
            open: false,
            title: String::new(),
            text: String::new(),
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker, title: &str, text: &str) {
        self.title.clear();
        self.title.push_str(title);
        self.text.clear();
        self.text.push_str(text);
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        if ! self.open { return; }

        if create_dialog_window(sys_dialogs, ui, self.id, Self::WINDOW_WIDTH, &self.title, |ui| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label(&self.text);
            });
            if ui.button("OK").clicked() {
                ui.close();
            }
        }).should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
    }
}
