use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{PropFont, DataAssetId, GenericAsset};

struct PropertiesDialog {
    image_changed: bool,
    open: bool,
    name: String,
    height: f32,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            height: 0.0,
        }
    }

    fn set_open(&mut self, prop_font: &PropFont) {
        self.name.clear();
        self.name.push_str(&prop_font.asset.name);
        self.height = prop_font.height as f32;
        self.open = true;
    }

    fn confirm(&mut self, prop_font: &mut PropFont) {
        prop_font.asset.name.clear();
        prop_font.asset.name.push_str(&self.name);

        let height = self.height as u32;
        if height != prop_font.height {
            let image = ImageCollection::from_asset(prop_font);
            image.resize(height * 2, height, PropFont::NUM_CHARS, &mut prop_font.data, 0x0c);
            prop_font.max_width = 2 * height;
            prop_font.height = height;
            for char_width in prop_font.char_widths.iter_mut() {
                if *char_width > (2 * height) as u8 {
                    *char_width = (2 * height) as u8;
                }
            }
            self.image_changed = true;
        }
    }

    fn show(&mut self, wc: &WindowContext, prop_font: &mut PropFont) -> bool {
        if egui::Modal::new(egui::Id::new("dlg_prop_font_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Prop Font Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", prop_font.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 4.0..=48.0).step_by(1.0));
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(prop_font);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}

pub struct PropFontEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: PropertiesDialog,
    force_reload_image: bool,
    selected_char: u32,
}

fn char_name(ch: char) -> String {
    if ch == ' ' {
        "(space)".to_string()
    } else if ch as u32 >= 127 {
        "DEL".to_string()
    } else {
        ch.to_string()
    }
}

impl PropFontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PropFontEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: PropertiesDialog::new(),
            force_reload_image: false,
            selected_char: 1,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, prop_font: &mut PropFont) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, prop_font) {
            self.force_reload_image = true;
        }

        let asset_id = prop_font.asset.id;
        let title = format!("{} - Prop Font", prop_font.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Proportional Font", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                self.properties_dialog.set_open(prop_font);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", prop_font.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let (image, texture) = ImageCollection::load_asset(prop_font, wc.tex_man, wc.egui.ctx, self.force_reload_image);

                let sel_char_width = prop_font.char_widths.get(self.selected_char as usize).map_or(1, |&v| v) as u32;

                ui.horizontal(|ui| {
                    ui.label("Selected:");
                    ui.add_space(5.0);
                    let cur_char = match char::from_u32(PropFont::FIRST_CHAR + self.selected_char) {
                        Some(ch) => char_name(ch),
                        None => " ".to_string(),
                    };
                    egui::ComboBox::from_id_salt(format!("editor_{}_sel_char", asset_id))
                        .selected_text(cur_char)
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for i in 0..PropFont::NUM_CHARS {
                                if let Some(ch) = char::from_u32(PropFont::FIRST_CHAR + i) {
                                    ui.selectable_value(&mut self.selected_char, i, char_name(ch));
                                }
                            }
                        });
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);

                    if ui.button("\u{2796}").clicked() &&
                        let Some(v) = prop_font.char_widths.get_mut(self.selected_char as usize) &&
                        *v > 1 {
                            *v -= 1;
                        }
                    ui.label(format!("{}", sel_char_width));
                    if ui.button("\u{2795}").clicked() &&
                        let Some(v) = prop_font.char_widths.get_mut(self.selected_char as usize) &&
                        *v < prop_font.max_width as u8 {
                            *v += 1;
                        }
                });
                ui.add_space(5.0);

                let (resp, canvas_to_image) = super::widgets::prop_font_image_editor(ui, texture, &image,
                                                                                     self.selected_char, sel_char_width);
                if let Some(pointer_pos) = resp.interact_pointer_pos() &&
                    canvas_to_image.from().contains(pointer_pos) {
                        let image_pos = canvas_to_image * pointer_pos;
                        let x = image_pos.x as i32;
                        let y = image_pos.y as i32;
                        if let Some(color) = if resp.dragged_by(egui::PointerButton::Primary) {
                            Some(0x00)
                        } else if resp.dragged_by(egui::PointerButton::Secondary) {
                            Some(0x0c)
                        } else {
                            None
                        } {
                            self.force_reload_image = image.set_pixel(&mut prop_font.data, x, y, self.selected_char, color);
                        }
                    }
            });
        });
    }
}
