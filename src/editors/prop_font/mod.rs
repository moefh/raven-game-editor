mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::data_asset::{PropFont, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::DataAssetEditor;
use super::widgets::PropFontEditorWidget;

pub struct PropFontEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl PropFontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PropFontEditor {
            asset: super::DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _prop_font: &mut PropFont) {
    }

    pub fn show(&mut self, wc: &mut WindowContext, prop_font: &mut PropFont) {
        self.dialogs.show(wc, &mut self.editor, prop_font);

        let title = format!("{} - Prop Font", prop_font.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        window.show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, prop_font);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, prop_font: &mut PropFont) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, prop_font) {
            editor.force_reload_image = true;
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    prop_font_editor: PropFontEditorWidget,
    force_reload_image: bool,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            prop_font_editor: PropFontEditorWidget::new().with_selected_char('@' as u32 - PropFont::FIRST_CHAR),
            force_reload_image: false,
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, prop_font: &mut PropFont) {
        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Prop Font", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(prop_font);
                        }
                    });
                });
            });
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes", prop_font.data_size()));
        });

        // body:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Selected:");
                ui.add_space(5.0);
                let cur_char = match char::from_u32(PropFont::FIRST_CHAR + self.prop_font_editor.selected_char) {
                    Some(ch) => Self::char_name(ch),
                    None => " ".to_string(),
                };
                egui::ComboBox::from_id_salt(format!("editor_{}_sel_char", self.asset_id))
                    .selected_text(cur_char)
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        for i in 0..PropFont::NUM_CHARS {
                            if let Some(ch) = char::from_u32(PropFont::FIRST_CHAR + i) {
                                ui.selectable_value(&mut self.prop_font_editor.selected_char, i, Self::char_name(ch));
                            }
                        }
                    });
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.button("\u{2796}").clicked() &&
                    let Some(v) = prop_font.char_widths.get_mut(self.prop_font_editor.selected_char as usize) &&
                    *v > 1 {
                        *v -= 1;
                    }

                ui.label(format!("{}", self.prop_font_editor.get_selected_char_width(prop_font)));
                if ui.button("\u{2795}").clicked() &&
                    let Some(v) = prop_font.char_widths.get_mut(self.prop_font_editor.selected_char as usize) &&
                    *v < prop_font.max_width as u8 {
                        *v += 1;
                    }
            });
            ui.add_space(5.0);

            if self.prop_font_editor.show(ui, wc, prop_font) {
                self.force_reload_image = true;
            }
        });
    }
}
