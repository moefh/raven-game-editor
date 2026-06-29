use super::{
    SysDialogs,
    AppWindowTracker,
    create_dialog_window,
};

pub enum ConfirmationDialogResult {
    None,
    Yes,
    No,
}

pub struct ConfirmationDialog {
    id: egui::Id,
    open: bool,
    title: String,
    text: String,
    yes_label: String,
    no_label: String,
}

impl ConfirmationDialog {
    const WINDOW_WIDTH: f32 = 350.0;

    pub fn new() -> Self {
        ConfirmationDialog {
            id: egui::Id::new("dlg_confirmation"),
            open: false,
            title: String::new(),
            text: String::new(),
            yes_label: String::new(),
            no_label: String::new(),
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker, title: &str, text: &str, yes_label: &str, no_label: &str) {
        self.title.clear();
        self.title.push_str(title);
        self.text.clear();
        self.text.push_str(text);
        self.yes_label.clear();
        self.yes_label.push_str(yes_label);
        self.no_label.clear();
        self.no_label.push_str(no_label);
        self.open = true;
        wt.set_dialog_open(self.id, self.open);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> ConfirmationDialogResult {
        if ! self.open { return ConfirmationDialogResult::No; }

        let resp = create_dialog_window(sys_dialogs, ui, self.id, Self::WINDOW_WIDTH, &self.title, |ui| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label(&self.text);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button(&self.yes_label).clicked() {
                    ui.close();
                    ConfirmationDialogResult::Yes
                } else if ui.button(&self.no_label).clicked() {
                    ui.close();
                    ConfirmationDialogResult::No
                } else {
                    ConfirmationDialogResult::None
                }
            }).inner
        });
        if resp.should_close() {
            self.open = false;
            wt.set_dialog_open(self.id, self.open);
        }
        resp.inner
    }
}
