use super::{
    SysDialogs,
    DialogResult,
    AppWindowTracker,
    create_dialog_window,
};

pub struct ConfirmExitDialog {
    id: egui::Id,
    open: bool,
}

impl ConfirmExitDialog {
    const WINDOW_WIDTH: f32 = 350.0;

    pub fn new() -> Self {
        ConfirmExitDialog {
            id: egui::Id::new("dlg_exit_confirmation"),
            open: false,
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker) {
        self.open = true;
        wt.set_dialog_open(self.id, self.open);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> DialogResult {
        if ! self.open { return DialogResult::None; }

        let resp = create_dialog_window(sys_dialogs, ui, self.id, Self::WINDOW_WIDTH, "Confirm Exit", |ui| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label("The editor has unsaved changes.");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Discard Changes").clicked() {
                    ui.close();
                    DialogResult::Yes
                } else if ui.button("Don't Close").clicked() {
                    ui.close();
                    DialogResult::No
                } else {
                    DialogResult::None
                }
            }).inner
        });
        if resp.should_close() {
            self.open = false;
            wt.set_dialog_open(self.id, self.open);
            if resp.inner == DialogResult::None {
                // closing because the user clicked outside the window
                return DialogResult::Cancel;
            }
        }
        resp.inner
    }
}
