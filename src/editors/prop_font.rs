use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{PropFont, DataAssetId, GenericAsset};

pub struct PropFontEditor {
    pub asset: super::DataAssetEditor,
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
            force_reload_image: false,
            selected_char: 1,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, prop_font: &mut PropFont) {
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
                                //...
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

                    if ui.button("\u{2796}").clicked() {
                        prop_font.char_widths.get_mut(self.selected_char as usize).map(|v| {
                            if *v > 1 { *v -= 1; }
                        });
                    }
                    ui.label(format!("{}", sel_char_width));
                    if ui.button("\u{2795}").clicked() {
                        let max_width = prop_font.max_width as u8;
                        prop_font.char_widths.get_mut(self.selected_char as usize).map(|v| {
                            if *v < max_width { *v += 1; }
                        });
                    }
                });

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
