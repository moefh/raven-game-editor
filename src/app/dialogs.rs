use super::IMAGES;

pub enum ConfirmationDialogResult {
    None,
    Yes,
    No,
}

pub struct AppDialogs {
    pub about_open: bool,
    pub message_box_open: bool,
    pub confirmation_dialog_open: bool,
    pub message_box_title: String,
    pub message_box_text: String,
    pub confirmation_dialog_title: String,
    pub confirmation_dialog_text: String,
    pub confirmation_dialog_yes: String,
    pub confirmation_dialog_no: String,
}

impl AppDialogs {
    pub fn new() -> Self {
        AppDialogs {
            about_open: false,
            message_box_open: false,
            confirmation_dialog_open: false,
            message_box_title: String::new(),
            message_box_text: String::new(),
            confirmation_dialog_title: String::new(),
            confirmation_dialog_text: String::new(),
            confirmation_dialog_yes: String::new(),
            confirmation_dialog_no: String::new(),
        }
    }

    pub fn open_message_box(&mut self, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box_open = true;
        self.message_box_text = text.as_ref().to_owned();
        self.message_box_title = title.as_ref().to_owned();
    }

    pub fn open_confirmation_dialog(&mut self, title: impl AsRef<str>, text: impl AsRef<str>, yes: impl AsRef<str>, no: impl AsRef<str>) {
        self.confirmation_dialog_open = true;

        self.confirmation_dialog_text.clear();
        self.confirmation_dialog_text.push_str(text.as_ref());

        self.confirmation_dialog_title.clear();
        self.confirmation_dialog_title.push_str(title.as_ref());

        self.confirmation_dialog_yes.clear();
        self.confirmation_dialog_yes.push_str(yes.as_ref());

        self.confirmation_dialog_no.clear();
        self.confirmation_dialog_no.push_str(no.as_ref());
    }

    pub fn open_about(&mut self) {
        self.about_open = true;
    }

    pub fn show_about(&mut self, ctx: &egui::Context, sys_dialogs: &super::SysDialogs) {
        if show_about_dialog(ctx, sys_dialogs).should_close() {
            self.about_open = false;
        }
    }

    pub fn show_message_box(&mut self, ctx: &egui::Context, sys_dialogs: &super::SysDialogs) {
        if show_message_box(ctx, sys_dialogs, &self.message_box_title, &self.message_box_text).should_close() {
            self.message_box_open = false;
        }
    }

    pub fn show_confirmation_dialog(&mut self, ctx: &egui::Context, sys_dialogs: &super::SysDialogs) -> ConfirmationDialogResult {
        let response = show_confirmation_dialog(ctx, sys_dialogs, &self.confirmation_dialog_title, &self.confirmation_dialog_text,
                                                &self.confirmation_dialog_yes, &self.confirmation_dialog_no);
        if response.should_close() {
            self.confirmation_dialog_open = false;
        }
        response.inner
    }
}

pub fn show_about_dialog(ctx: &egui::Context, sys_dialogs: &super::SysDialogs) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new("dlg_about")).show(ctx, |ui| {
        sys_dialogs.block_ui(ui);
        ui.set_width(400.0);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            ui.heading("About Raven Game Editor");
            ui.separator();
            ui.add_space(12.0);
            ui.add(egui::Image::new(IMAGES.pico).max_width(32.0).max_height(32.0));
            ui.add_space(16.0);
            ui.label("Copyright (C) 2025 MoeFH");
            ui.add_space(16.0);
            ui.label("Source code:");
            ui.add_space(5.0);
            ui.hyperlink("https://github.com/moefh/raven-game-editor/");
            ui.add_space(20.0);
            if ui.button("Close").clicked() {
                ui.close();
            }
        });
    })
}

pub fn show_message_box(ctx: &egui::Context, sys_dialogs: &super::SysDialogs, title: &str, text: &str) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new("dlg_message_box")).show(ctx, |ui| {
        sys_dialogs.block_ui(ui);
        ui.set_width(350.0);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            ui.heading(title);
            ui.separator();
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label(text);
            });
            if ui.button("OK").clicked() {
                ui.close();
            }
        });
    })
}

pub fn show_confirmation_dialog(ctx: &egui::Context, sys_dialogs: &super::SysDialogs, title: &str, text: &str,
                                yes_button: &str, no_button: &str) -> egui::ModalResponse<ConfirmationDialogResult> {
    egui::Modal::new(egui::Id::new("dlg_message_box")).show(ctx, |ui| {
        sys_dialogs.block_ui(ui);
        ui.set_width(350.0);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            ui.heading(title);
            ui.separator();
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.label(text);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button(yes_button).clicked() {
                    ui.close();
                    ConfirmationDialogResult::Yes
                } else if ui.button(no_button).clicked() {
                    ui.close();
                    ConfirmationDialogResult::No
                } else {
                    ConfirmationDialogResult::None
                }
            }).inner
        }).inner
    })
}
