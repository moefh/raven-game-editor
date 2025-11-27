mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::data_asset::{Font, DataAssetId, ImageCollectionAsset, GenericAsset};

use properties::PropertiesDialog;
use super::DataAssetEditor;
use super::widgets::ImageEditorWidget;

pub struct FontEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl FontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        FontEditor {
            asset: DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, asset: &mut impl ImageCollectionAsset) {
        self.editor.image_editor.drop_selection(asset);
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) {
        self.dialogs.show(wc, &mut self.editor, font);

        let title = format!("{} - Font", font.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        let (min_size, default_size) = super::calc_image_editor_window_size(font);
        window.min_size(min_size).default_size(default_size).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, font);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
}

impl Dialogs {
    pub fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, font: &mut Font) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, font) {
            editor.force_reload_image = true;
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    force_reload_image: bool,
    image_editor: ImageEditorWidget,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            force_reload_image: false,
            image_editor: ImageEditorWidget::new().with_selected_image('@' as u32 - Font::FIRST_CHAR),
        }
    }

    pub fn is_on_top(&self, wc: &WindowContext) -> bool {
        match wc.top_editor_asset_id {
            Some(top_id) => top_id == self.asset_id,
            None => false,
        }
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, font: &mut Font) {
        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Font", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(font);
                        }
                    });
                });
            });
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes", font.data_size()));
        });

        // body:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Selected:");
                ui.add_space(5.0);
                let cur_char = match char::from_u32(Font::FIRST_CHAR + self.image_editor.get_selected_image()) {
                    Some(ch) => Self::char_name(ch),
                    None => " ".to_string(),
                };
                egui::ComboBox::from_id_salt(format!("editor_{}_sel_char", self.asset_id))
                    .selected_text(cur_char)
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        for i in 0..Font::NUM_CHARS {
                            if let Some(ch) = char::from_u32(Font::FIRST_CHAR + i) {
                                let mut sel_image = self.image_editor.get_selected_image();
                                ui.selectable_value(&mut sel_image, i, Self::char_name(ch));
                                self.image_editor.set_selected_image(sel_image, font);
                            }
                        }
                    });
            });
            ui.add_space(5.0);

            let colors = (Font::FG_COLOR, Font::BG_COLOR);
            self.image_editor.show(ui, wc, font, colors);
        });

        // keyboard:
        if self.is_on_top(wc) {
            self.image_editor.handle_keyboard(ui, font);
        }
    }
}
