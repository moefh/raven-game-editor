use super::AppWindow;
use super::super::WindowContext;

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

    pub fn show(&mut self, wc: &WindowContext, vga_sync_bits: &mut u8, project_prefix: &mut String) {
        let default_rect = self.base.default_rect(wc, 300.0, 200.0);
        let response = self.base.create_window(wc, "Project Properties", default_rect).show(wc.egui.ctx, |ui| {
            egui::Grid::new("project_properties_grid")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("VGA sync bits:");
                    egui::ComboBox::from_id_salt("project_properties_vga_sync_bits_combo")
                        .selected_text(Self::VGA_SYNC_BITS_OPTIONS[((*vga_sync_bits >> 6) & 0x3) as usize])
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for i in 0..4 {
                                ui.selectable_value(vga_sync_bits, i<<6, Self::VGA_SYNC_BITS_OPTIONS[i as usize]);
                            }
                        });
                    ui.end_row();

                    ui.label("Identifier prefix:");
                    ui.add_sized([150.0, ui.available_height()], egui::TextEdit::singleline(project_prefix));
                    ui.end_row();
                });
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Close").clicked() {
                    ui.close();
                }
            });
        });
        if let Some(response) = response && response.response.should_close() {
            self.base.open = false;
        }
    }
}
