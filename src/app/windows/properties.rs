const VGA_SYNC_BITS_OPTIONS: &[&str] = &[
    "0x00 (00)",
    "0x40 (01)",
    "0x80 (10)",
    "0xc0 (11)",
];

pub fn show_project_properties(ctx: &egui::Context, window_space: egui::Rect, window_open: &mut bool,
                               vga_sync_bits: &mut u8, project_prefix: &mut String) {
    let window_id = egui::Id::new("project_properties");
    let default_rect = egui::Rect {
        min: egui::Pos2 {
            x : window_space.min.x + 10.0,
            y : window_space.min.y + 10.0,
        },
        max: egui::Pos2 {
            x: 400.0,
            y: 300.0,
        }
    };
    let mut close_window = false;
    egui::Window::new("Project Properties")
        .id(window_id)
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
        .open(window_open).show(ctx, |ui| {
            egui::Grid::new("project_properties_grid")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("VGA sync bits:");
                    egui::ComboBox::from_id_salt("project_properties_vga_sync_bits_combo")
                        .selected_text(VGA_SYNC_BITS_OPTIONS[((*vga_sync_bits >> 6) & 0x3) as usize])
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for i in 0..4 {
                                ui.selectable_value(vga_sync_bits, i<<6, VGA_SYNC_BITS_OPTIONS[i as usize]);
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
                    close_window = true;
                }
            });
        });
    if close_window {
        *window_open = false;
    }
}
