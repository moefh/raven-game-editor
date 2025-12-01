use super::IMAGES;

use super::{SysDialogs, AppWindowTracker};

pub enum ConfirmationDialogResult {
    None,
    Yes,
    No,
}

struct AboutDialog {
    id: egui::Id,
    open: bool,
}

struct MessageBoxDialog {
    id: egui::Id,
    open: bool,
    title: String,
    text: String,
}

impl MessageBoxDialog {
    fn new() -> Self {
        MessageBoxDialog {
            id: egui::Id::new("dlg_message_box"),
            open: false,
            title: String::new(),
            text: String::new(),
        }
    }

    fn set_open(&mut self, wt: &mut AppWindowTracker, title: &str, text: &str) {
        self.title.clear();
        self.title.push_str(title);
        self.text.clear();
        self.text.push_str(text);
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    fn show(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        if ! self.open { return; }

        if egui::Modal::new(self.id).show(ctx, |ui| {
            sys_dialogs.block_ui(ui);
            ui.set_width(350.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading(&self.title);
                ui.separator();
                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    ui.label(&self.text);
                });
                if ui.button("OK").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
    }
}

struct ConfirmationDialog {
    id: egui::Id,
    open: bool,
    title: String,
    text: String,
    yes_label: String,
    no_label: String,
}

impl ConfirmationDialog {
    fn new() -> Self {
        ConfirmationDialog {
            id: egui::Id::new("dlg_confirmation"),
            open: false,
            title: String::new(),
            text: String::new(),
            yes_label: String::new(),
            no_label: String::new(),
        }
    }

    fn set_open(&mut self, wt: &mut AppWindowTracker, title: &str, text: &str, yes_label: &str, no_label: &str) {
        self.title.clear();
        self.title.push_str(title);
        self.text.clear();
        self.text.push_str(text);
        self.yes_label.clear();
        self.yes_label.push_str(yes_label);
        self.no_label.clear();
        self.no_label.push_str(no_label);
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    fn show(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> ConfirmationDialogResult {
        if ! self.open { return ConfirmationDialogResult::No; }

        let resp = egui::Modal::new(self.id).show(ctx, |ui| {
            sys_dialogs.block_ui(ui);
            ui.set_width(350.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading(&self.title);
                ui.separator();
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
            }).inner
        });
        if resp.should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
        resp.inner
    }
}

impl AboutDialog {
    fn new() -> Self {
        AboutDialog {
            id: egui::Id::new("dlg_about"),
            open: false,
        }
    }

    fn set_open(&mut self, wt: &mut AppWindowTracker) {
        self.open = true;
        wt.set_open(self.id, self.open);
    }

    fn show(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        if ! self.open { return; }

        if egui::Modal::new(self.id).show(ctx, |ui| {
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
        }).should_close() {
            self.open = false;
            wt.set_open(self.id, self.open);
        }
    }
}

pub struct AppDialogs {
    about: AboutDialog,
    message_box: MessageBoxDialog,
    confirmation: ConfirmationDialog,
}

impl AppDialogs {
    pub fn new() -> Self {
        AppDialogs {
            about: AboutDialog::new(),
            message_box: MessageBoxDialog::new(),
            confirmation: ConfirmationDialog::new(),
        }
    }

    pub fn open_message_box(&mut self, wt: &mut AppWindowTracker, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box.set_open(wt, title.as_ref(), text.as_ref());
    }

    pub fn open_confirmation_dialog(&mut self, wt: &mut AppWindowTracker, title: impl AsRef<str>,
                                    text: impl AsRef<str>, yes: impl AsRef<str>, no: impl AsRef<str>) {
        self.confirmation.set_open(wt, title.as_ref(), text.as_ref(), yes.as_ref(), no.as_ref());
    }

    pub fn open_about(&mut self, wt: &mut AppWindowTracker) {
        self.about.set_open(wt);
    }

    pub fn show_about(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        self.about.show(ctx, wt, sys_dialogs);
    }

    pub fn show_message_box(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) {
        self.message_box.show(ctx, wt, sys_dialogs);
    }

    pub fn show_confirmation_dialog(&mut self, ctx: &egui::Context, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs)
                                    -> ConfirmationDialogResult {
        self.confirmation.show(ctx, wt, sys_dialogs)
    }
}
