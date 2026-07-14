use super::{
    SysDialogs,
    DialogResult,
    AppWindowTracker,
    create_dialog_window,
};

pub struct TextInputDialog {
    id: egui::Id,
    open: bool,
    title: String,
    prompt: String,
    input: String,
    yes_label: String,
    no_label: String,
}

impl TextInputDialog {
    const WINDOW_WIDTH: f32 = 350.0;

    pub fn new() -> Self {
        TextInputDialog {
            id: egui::Id::new("dlg_text_input"),
            open: false,
            title: String::new(),
            prompt: String::new(),
            input: String::new(),
            yes_label: String::new(),
            no_label: String::new(),
        }
    }

    pub fn set_open(&mut self, wt: &mut AppWindowTracker, title: &str, prompt: &str, input: &str, yes_label: &str, no_label: &str) {
        self.title.clear();
        self.title.push_str(title);
        self.prompt.clear();
        self.prompt.push_str(prompt);
        self.input.clear();
        self.input.push_str(input);
        self.yes_label.clear();
        self.yes_label.push_str(yes_label);
        self.no_label.clear();
        self.no_label.push_str(no_label);
        self.open = true;
        wt.set_dialog_open(self.id, self.open);
    }

    pub fn get_input(&mut self) -> String {
        std::mem::take(&mut self.input)
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> DialogResult {
        if ! self.open { return DialogResult::None; }

        let resp = create_dialog_window(sys_dialogs, ui, self.id, Self::WINDOW_WIDTH, &self.title, |ui| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.prompt);
                    ui.text_edit_singleline(&mut self.input);
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button(&self.yes_label).clicked() {
                    ui.close();
                    DialogResult::Yes
                } else if ui.button(&self.no_label).clicked() {
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
