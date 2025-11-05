use crate::image_table::IMAGES;

pub fn show_about_dialog(ctx: &egui::Context) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new("dlg_about")).show(ctx, |ui| {
        ui.set_width(250.0);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            ui.heading("About Raven Game Editor");
            ui.add_space(16.0);
            ui.add(egui::Image::new(IMAGES.pico).max_width(32.0).max_height(32.0));
            ui.add_space(16.0);
            ui.label("Copyright (C) 2025 MoeFH");
            ui.add_space(16.0);
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
            ui.add_space(16.0);
            ui.label(text);
            ui.add_space(16.0);
            if ui.button("OK").clicked() {
                ui.close();
            }
        });
    })
}
