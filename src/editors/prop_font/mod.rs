mod properties;

use crate::misc::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollection, ImagePixels, TextureSlot};
use crate::data_asset::{PropFont, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::DataAssetEditor;
use super::widgets::{PropFontEditorWidget, FontViewWidget, FontPainter};

use egui::{Pos2, Rect};

impl FontPainter for PropFont {
    fn measure(&self, height: f32, text: &str) -> f32 {
        let zoom = height / self.height as f32;
        text.chars().fold(0.0, |size, ch| {
            let char_index = (ch as u32).saturating_sub(PropFont::FIRST_CHAR);
            size + zoom * (self.char_widths.get(char_index as usize).copied().unwrap_or(0) as f32 + 1.0)
        })
    }

    fn paint_char(&self, ui: &mut egui::Ui, wc: &mut WindowContext, ch: char, pos: egui::Pos2, height: f32) -> f32 {
        if (ch as u32) < PropFont::FIRST_CHAR { return 0.0; }
        let char_index = (ch as u32).saturating_sub(PropFont::FIRST_CHAR);
        let char_width = self.char_widths.get(char_index as usize).copied().unwrap_or(0) as f32;
        let zoom = height / self.height as f32;
        let width = zoom * char_width;

        let texture = self.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        let image_size = self.get_item_size();
        let rect = egui::Rect {
            min: pos,
            max: pos + egui::Vec2::new(width, height),
        };
        let uv = Rect {
            min: Pos2::new(0.0, char_index as f32 / PropFont::NUM_CHARS as f32),
            max: Pos2::new(char_width / self.max_width as f32, (char_index+1) as f32 / PropFont::NUM_CHARS as f32),
        };
        egui::Image::from_texture((texture.id(), image_size)).uv(uv).paint_at(ui, rect);
        width + zoom
    }
}

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
        window.min_size([350.0, 250.0]).default_size([400.0, 350.0]).show(wc.egui.ctx, |ui| {
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
            editor.prop_font_editor.image_changed = true;
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    prop_font_editor: PropFontEditorWidget,
    font_view: FontViewWidget,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            prop_font_editor: PropFontEditorWidget::new().with_selected_char('@' as u32 - PropFont::FIRST_CHAR),
            font_view: FontViewWidget::new(),
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

    fn export_dlg_id(pfont: &PropFont) -> String {
        format!("editor_{}_export_pfont", pfont.asset.id)
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, prop_font: &mut PropFont) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(Self::export_dlg_id(prop_font)) &&
            let Err(e) = ImagePixels::save_prop_font_png(&filename, prop_font) {
                wc.open_message_box("Error Exporting", format!("Error exporting prop font to {}:\n{}", filename.display(), e));
            }

        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Prop Font", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            wc.sys_dialogs.save_file(
                                Some(wc.egui.window),
                                Self::export_dlg_id(prop_font),
                                "Export Prop Font",
                                &[
                                    ("PNG files (*.png)", &["png"]),
                                    ("All files (*.*)", &["*"]),
                                ]
                            );
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, prop_font);
                        }
                    });
                });
            });
        });

        // font test:
        egui::TopBottomPanel::top(format!("editor_panel_{}_font_test", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("Edit:");
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

                ui.separator();

                ui.label("Sample:");
                ui.text_edit_singleline(&mut self.font_view.text);
            });

            ui.add_space(8.0);
            self.font_view.show(ui, wc, prop_font);
            ui.add_space(2.0);
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes [height: {}]", prop_font.data_size(), prop_font.height));
        });

        // body:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.prop_font_editor.show(ui, wc, prop_font);
        });
    }
}
