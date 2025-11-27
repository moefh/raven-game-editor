mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::data_asset::{Font, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::widgets::ImageEditorWidget;

pub struct FontEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: PropertiesDialog,
    force_reload_image: bool,
    image_editor: ImageEditorWidget,
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
            image_editor: ImageEditorWidget::new().with_selected_image('@' as u32 - Font::FIRST_CHAR),
        }
    }

    pub fn prepare_for_saving(&mut self, font: &mut Font) {
        self.image_editor.drop_selection(font);
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
                ui.horizontal(|ui| {
                    ui.label("Selected:");
                    ui.add_space(5.0);
                    let cur_char = match char::from_u32(Font::FIRST_CHAR + self.image_editor.selected_image) {
                        Some(ch) => char_name(ch),
                        None => " ".to_string(),
                    };
                    egui::ComboBox::from_id_salt(format!("editor_{}_sel_char", asset_id))
                        .selected_text(cur_char)
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for i in 0..Font::NUM_CHARS {
                                if let Some(ch) = char::from_u32(Font::FIRST_CHAR + i) {
                                    ui.selectable_value(&mut self.image_editor.selected_image, i, char_name(ch));
                                }
                            }
                        });
                });
                ui.add_space(5.0);

                let colors = (Font::FG_COLOR, Font::BG_COLOR);
                self.image_editor.show(ui, wc, font, colors);
            });
        });
    }
}
