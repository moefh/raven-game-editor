use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{Font, DataAssetId, GenericAsset};

struct PropertiesDialog {
    image_changed: bool,
    open: bool,
    name: String,
    width: f32,
    height: f32,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    fn set_open(&mut self, font: &Font) {
        self.name.clear();
        self.name.push_str(&font.asset.name);
        self.width = font.width as f32;
        self.height = font.height as f32;
        self.open = true;
    }

    fn confirm(&mut self, font: &mut Font) {
        font.asset.name.clear();
        font.asset.name.push_str(&self.name);

        let width = self.width as u32;
        let height = self.height as u32;
        if width != font.width || height != font.height {
            let image = ImageCollection::from_asset(font);
            image.resize(width, height, Font::NUM_CHARS, &mut font.data, 0x0c);
            font.width = width;
            font.height = height;
            self.image_changed = true;
        }
    }

    fn show(&mut self, wc: &WindowContext, font: &mut Font) -> bool {
        if egui::Modal::new(egui::Id::new("dlg_font_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Font Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", font.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 4.0..=48.0).step_by(1.0));
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
                        self.confirm(font);
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

pub struct FontEditor {
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

impl FontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        FontEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: PropertiesDialog::new(),
            force_reload_image: false,
            selected_char: 1,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, font) {
            self.force_reload_image = true;
        }

        let asset_id = font.asset.id;
        let title = format!("{} - Font", font.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        window.open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Font", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                self.properties_dialog.set_open(font);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", font.data_size()));
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let (image, texture) = ImageCollection::load_asset(font, wc.tex_man, wc.egui.ctx, self.force_reload_image);

                ui.horizontal(|ui| {
                    ui.label("Selected:");
                    ui.add_space(5.0);
                    let cur_char = match char::from_u32(Font::FIRST_CHAR + self.selected_char) {
                        Some(ch) => char_name(ch),
                        None => " ".to_string(),
                    };
                    egui::ComboBox::from_id_salt(format!("editor_{}_sel_char", asset_id))
                        .selected_text(cur_char)
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for i in 0..Font::NUM_CHARS {
                                if let Some(ch) = char::from_u32(Font::FIRST_CHAR + i) {
                                    ui.selectable_value(&mut self.selected_char, i, char_name(ch));
                                }
                            }
                        });
                });
                ui.add_space(5.0);

                let (resp, canvas_to_image) = super::widgets::image_editor(ui, texture, &image, self.selected_char);
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
                            self.force_reload_image = image.set_pixel(&mut font.data, x, y, self.selected_char, color);
                        }
                    }
            });
        });
    }
}
