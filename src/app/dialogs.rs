use super::IMAGES;

pub struct AppDialogs {
    pub about_open: bool,
    pub message_box_open: bool,
    pub message_box_title: String,
    pub message_box_text: String,
}

impl AppDialogs {
    pub fn new() -> Self {
        AppDialogs {
            about_open: false,
            message_box_open: false,
            message_box_title: "".to_owned(),
            message_box_text: "".to_owned(),
        }
    }

    pub fn open_message_box(&mut self, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box_open = true;
        self.message_box_text = text.as_ref().to_owned();
        self.message_box_title = title.as_ref().to_owned();
    }

    pub fn open_about(&mut self) {
        self.about_open = true;
    }

    pub fn show_about(&mut self, ctx: &egui::Context) {
        if show_about_dialog(ctx).should_close() {
            self.about_open = false;
        }
    }

    pub fn show_message_box(&mut self, ctx: &egui::Context) {
        if show_message_box(ctx, &self.message_box_title, &self.message_box_text).should_close() {
            self.message_box_open = false;
        }
    }
}

pub fn show_about_dialog(ctx: &egui::Context) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new("dlg_about")).show(ctx, |ui| {
        ui.set_width(400.0);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            ui.heading("About Raven Game Editor");
            ui.separator();
            ui.add_space(12.0);
            ui.add(egui::Image::new(IMAGES.pico).max_width(32.0).max_height(32.0));
            ui.add_space(16.0);
            ui.label("Copyright (C) 2025 MoeFH");
            ui.add_space(8.0);
            ui.label("Source code released under the MIT license at");
            ui.add_space(5.0);
            ui.hyperlink("https://github.com/moefh/raven-game-editor/");
            ui.add_space(20.0);
            if ui.button("Close").clicked() {
                ui.close();
            }
        });
    })
}

pub fn show_message_box(ctx: &egui::Context, title: &str, text: &str) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new("dlg_message_box")).show(ctx, |ui| {
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
