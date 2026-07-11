mod properties;
mod import;

use crate::misc::IMAGES;
use crate::app::{
    menu_item,
    menu_item_no_image,
    WindowContext,
    SysDialogResponse,
};
use crate::image::{
    ImageCollection,
    ImagePixels,
    TextureSlot,
};
use crate::data_asset::{
    PropFont,
    DataAssetId,
    GenericAsset,
};

use properties::PropertiesDialog;
use import::ImportDialog;
use super::AssetEditorBase;
use super::widgets::{
    PropFontEditorWidget,
    FontViewWidget,
    FontPainter,
};

use egui::{Pos2, Rect};

impl FontPainter for PropFont {
    fn font_height(&self) -> f32 {
        self.height as f32
    }

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
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl PropFontEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PropFontEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _prop_font: &mut PropFont) {
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, prop_font: &PropFont, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, prop_font.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", prop_font.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            ui.label(format!("{} bytes [height: {}]{}", prop_font.data_size(), prop_font.height, dirty));
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, prop_font: &mut PropFont) {
        self.dialogs.show(wc, &mut self.editor, prop_font);

        self.base.show_window(wc, prop_font, [450.0, 300.0], [450.0, 400.0], |ui, wc, prop_font, base| {
            Self::show_footer(ui, wc, prop_font, base);
            self.editor.show(ui, wc, &mut self.dialogs, prop_font);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    import_dialog: ImportDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            import_dialog: ImportDialog::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, prop_font: &mut PropFont) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, prop_font) {
            editor.prop_font_editor.set_image_changed();
        }
        if self.import_dialog.open && self.import_dialog.show(wc, prop_font) {
            editor.prop_font_editor.selected_char = 0;
            editor.prop_font_editor.set_image_changed();
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    prop_font_editor: PropFontEditorWidget,
    font_view: FontViewWidget,
    export_sys_dlg_id: String,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            prop_font_editor: PropFontEditorWidget::new().with_selected_char('@' as u32 - PropFont::FIRST_CHAR),
            font_view: FontViewWidget::new(),
            export_sys_dlg_id: format!("editor_{}_export_pfont", asset_id),
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

    fn shift_image(&mut self, prop_font: &mut PropFont, dx: i32, dy: i32) {
        prop_font.shift_pixels(self.prop_font_editor.selected_char, dx, dy, true, PropFont::BG_COLOR);
        self.prop_font_editor.set_image_changed();
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, prop_font: &mut PropFont) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Prop Font", |ui| {
                    if ui.add(menu_item(IMAGES.import, " Import...")).clicked() {
                        dialogs.import_dialog.set_open(wc, prop_font);
                    }
                    if ui.add(menu_item(IMAGES.export, " Export...")).clicked() {
                        wc.sys_dialogs.save_file(
                            Some(wc.egui.window),
                            self.export_sys_dlg_id.clone(),
                            "prop_font",
                            "Export Prop Font",
                            &[
                                ("PNG files (*.png)", &["png"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, prop_font);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add(menu_item_no_image(" Invert colors")).clicked() {
                        for color in prop_font.data.iter_mut() {
                            *color = if *color == PropFont::FG_COLOR { PropFont::BG_COLOR } else { PropFont::FG_COLOR };
                        }
                        self.prop_font_editor.set_image_changed();
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.arrow_up, " Shift up")).clicked() {
                        self.shift_image(prop_font, 0, -1);
                    }
                    if ui.add(menu_item(IMAGES.arrow_down, " Shift down")).clicked() {
                        self.shift_image(prop_font, 0, 1);
                    }
                    if ui.add(menu_item(IMAGES.arrow_left, " Shift left")).clicked() {
                        self.shift_image(prop_font, -1, 0);
                    }
                    if ui.add(menu_item(IMAGES.arrow_right, " Shift right")).clicked() {
                        self.shift_image(prop_font, 1, 0);
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, prop_font: &mut PropFont) {
        egui::Panel::top(format!("editor_panel_{}_pfont_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(3.0, 0.0);

                ui.label("Edit:");
                ui.add_space(5.0);
                if ui.button("<").on_hover_text("Previous character").clicked() {
                    self.prop_font_editor.selected_char = self.prop_font_editor.selected_char.saturating_sub(1);
                }
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
                if ui.button(">").on_hover_text("Next character").clicked() {
                    self.prop_font_editor.selected_char = (self.prop_font_editor.selected_char + 1).min(PropFont::NUM_CHARS-1);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.button("\u{2796}").on_hover_text("Decrease width").clicked() &&
                    let Some(v) = prop_font.char_widths.get_mut(self.prop_font_editor.selected_char as usize) &&
                    *v > 1 {
                        *v -= 1;
                    }
                ui.label(format!("{}", self.prop_font_editor.get_selected_char_width(prop_font)));
                if ui.button("\u{2795}").on_hover_text("Increase width").clicked() &&
                    let Some(v) = prop_font.char_widths.get_mut(self.prop_font_editor.selected_char as usize) &&
                    *v < prop_font.max_width as u8 {
                        *v += 1;
                    }

                ui.add_space(15.0);

                if ui.add(egui::Button::image(IMAGES.arrow_up)).on_hover_text("Shift Up").clicked() {
                    self.shift_image(prop_font, 0, -1);
                }
                if ui.add(egui::Button::image(IMAGES.arrow_down)).on_hover_text("Shift Down").clicked() {
                    self.shift_image(prop_font, 0, 1);
                }
                if ui.add(egui::Button::image(IMAGES.arrow_left)).on_hover_text("Shift Left").clicked() {
                    self.shift_image(prop_font, -1, 0);
                }
                if ui.add(egui::Button::image(IMAGES.arrow_right)).on_hover_text("Shift Right").clicked() {
                    self.shift_image(prop_font, 1, 0);
                }
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_samplebar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, prop_font: &mut PropFont) {
        egui::Panel::top(format!("editor_panel_{}_pfont_sample", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("Sample:");
                egui::ComboBox::from_id_salt(format!("editor_{}_zoom_combo", prop_font.asset.id))
                    .selected_text(format!("{}x", self.font_view.zoom))
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        for z in 2..=10 {
                            ui.selectable_value(&mut self.font_view.zoom, z as f32, format!("{}x", z));
                        }
                    });
                ui.text_edit_singleline(&mut self.font_view.text);
            });
            ui.add_space(4.0);
            self.font_view.show(ui, wc, prop_font);
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, prop_font: &mut PropFont) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&self.export_sys_dlg_id) &&
            let Err(e) = ImagePixels::save_prop_font_png(&filename, prop_font) {
                wc.open_message_box("Error Exporting", format!("Error exporting prop font to {}:\n{}", filename.display(), e));
            }

        self.show_menubar(ui, wc, dialogs, prop_font);
        self.show_toolbar(ui, wc, prop_font);
        self.show_samplebar(ui, wc, prop_font);

        // body:
        egui::CentralPanel::default().show(ui, |ui| {
            self.prop_font_editor.show(ui, wc, prop_font);
        });
    }
}
