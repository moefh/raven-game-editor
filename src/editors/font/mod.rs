mod properties;

use crate::misc::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::{Font, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::AssetEditorBase;
use super::widgets::{ImageEditorWidget, FontViewWidget, FontPainter};

impl FontPainter for Font {
    fn measure(&self, height: f32, text: &str) -> f32 {
        let zoom = height / self.height as f32;
        text.chars().fold(0.0, |size, _ch| {
            size + zoom * self.width as f32
        })
    }

    fn paint_char(&self, ui: &mut egui::Ui, wc: &mut WindowContext, ch: char, pos: egui::Pos2, height: f32) -> f32 {
        if (ch as u32) < Font::FIRST_CHAR { return 0.0; }

        let zoom = height / self.height as f32;
        let width = zoom * self.width as f32;
        let texture = self.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        let image_size = self.get_item_size();
        let rect = egui::Rect {
            min: pos,
            max: pos + egui::Vec2::new(width, height),
        };
        egui::Image::from_texture((texture.id(), image_size)).uv(self.get_item_uv(ch as u32 - Font::FIRST_CHAR)).paint_at(ui, rect);
        width
    }
}

pub struct FontEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl FontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        FontEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, font: &mut Font) {
        self.editor.image_editor.drop_selection(font);
    }

    pub fn show(&mut self, wc: &mut WindowContext, font: &mut Font) {
        self.dialogs.show(wc, &mut self.editor, font);

        let title = format!("{} - Font", font.asset.name);
        let window = self.base.create_window(wc, &title);
        window.min_size([300.0, 250.0]).default_size([400.0, 350.0]).show(wc.egui.ctx, |ui| {
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
            editor.image_editor.set_undo_target(font);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    force_reload_image: bool,
    image_editor: ImageEditorWidget,
    font_view: FontViewWidget,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            force_reload_image: false,
            image_editor: ImageEditorWidget::new().with_selected_image('@' as u32 - Font::FIRST_CHAR),
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

    fn export_dlg_id(font: &Font) -> String {
        format!("editor_{}_export_font", font.asset.id)
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, font: &mut Font) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(Self::export_dlg_id(font)) &&
            let Err(e) = font.save_font_png(&filename, 16) {
                wc.open_message_box("Error Exporting", format!("Error exporting font to {}:\n{}", filename.display(), e));
            }

        // header:
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Font", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            wc.sys_dialogs.save_file(
                                Some(wc.egui.window),
                                Self::export_dlg_id(font),
                                "Export Font",
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
                            dialogs.properties_dialog.set_open(wc, font);
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

                ui.separator();

                ui.label("Sample:");
                ui.text_edit_singleline(&mut self.font_view.text);
            });

            ui.add_space(8.0);
            self.font_view.show(ui, wc, font);
            ui.add_space(2.0);
        });

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes [size: {}x{}]", font.data_size(), font.width, font.height));
        });

        // body:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let colors = (Font::FG_COLOR, Font::BG_COLOR);
            self.image_editor.show(ui, wc, font, colors);
        });

        // keyboard:
        if wc.is_editor_on_top(self.asset_id) {
            self.image_editor.handle_keyboard(ui, wc, font, Font::BG_COLOR);
        }
    }
}
