use super::AppWindow;
use super::super::{
    WindowContext,
    DataAssetStore,
};

pub struct PropertiesWindow {
    pub base: super::AppWindow,
}

impl PropertiesWindow {
    pub fn new(base: AppWindow) -> Self {
        PropertiesWindow {
            base,
        }
    }

    const VGA_SYNC_BITS_OPTIONS: &[&str] = &[
        "0x00 (00)",
        "0x40 (01)",
        "0x80 (10)",
        "0xc0 (11)",
    ];

    fn show_properties_grid(ui: &mut egui::Ui, wc: &mut WindowContext, store: &mut DataAssetStore) {
        egui::Grid::new("project_properties_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                // prefix
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Prefix:");
                });
                ui.add_sized([150.0, ui.available_height()], egui::TextEdit::singleline(&mut store.project_prefix));
                ui.end_row();

                // colors (bits per pixel, sync bits)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label("Colors:");
                });
                ui.vertical(|ui| {
                    let old_bits_per_pixel = store.vga_bits_per_pixel;
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut store.vga_bits_per_pixel, 8, "256 (8 bits)");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut store.vga_bits_per_pixel, 6, "64 (6 bits) with sync bits:");
                        ui.add_enabled_ui(store.vga_bits_per_pixel == 6, |ui| {
                            egui::ComboBox::from_id_salt("project_properties_vga_sync_bits_combo")
                                .selected_text(Self::VGA_SYNC_BITS_OPTIONS[((store.vga_sync_bits >> 6) & 0x3) as usize])
                                .width(50.0)
                                .show_ui(ui, |ui| {
                                    for i in 0..4 {
                                        ui.selectable_value(&mut store.vga_sync_bits, i<<6, Self::VGA_SYNC_BITS_OPTIONS[i as usize]);
                                    }
                                });
                        });
                    });
                    if old_bits_per_pixel != store.vga_bits_per_pixel {
                        wc.tex_man.set_bits_per_pixel(store.vga_bits_per_pixel);
                    }
                });
                ui.end_row();

                // tiles per world block
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Tiles per world block:");
                });
                ui.add(egui::DragValue::new(&mut store.tiles_per_world_block).speed(1.0).range(8..=32));
                ui.end_row();
            });
    }

    pub fn show(&mut self, wc: &mut WindowContext, store: &mut DataAssetStore) {
        let title = "Project Properties";
        let default_rect = self.base.default_rect(wc, 400.0, 150.0);
        let resp = self.base.create_window(wc, title, default_rect).resizable(false).show(wc.egui.ctx, |ui| {
            let action = AppWindow::show_title_bar(ui, title);
            egui::CentralPanel::default().show(ui, |ui| {
                Self::show_properties_grid(ui, wc, store);
            });
            action
        });
        self.base.run_window_action(resp);
    }
}
