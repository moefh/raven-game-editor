mod properties;
mod add_frames;
mod remove_frames;
mod import;
mod export;

use crate::misc::IMAGES;
use crate::image::{
    colors,
    ImageCollection,
    ImagePixels,
    TextureSlot,
    ImageRotation,
};
use crate::data_asset::{
    DataAssetId,
    Sprite,
    GenericAsset,
};

use super::{
    IMAGE_ZOOM_OPTIONS,
    ImageZoomOption,
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
};
use super::dialogs::CreateColorsetDialog;
use super::widgets::{
    ColorPickerWidget,
    ColorPickerResponse,
    ImagePickerWidget,
    ImageEditorWidget,
    ImageDrawingTool,
    ImageDisplay,
};
use super::super::menu_item;

use properties::PropertiesDialog;
use remove_frames::RemoveFramesDialog;
use add_frames::{AddFramesDialog, AddFramesAction};
use import::ImportDialog;
use export::ExportDialog;

pub struct SpriteEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(id),
        }
    }

    pub fn prepare_for_saving(&mut self, sprite: &mut Sprite) {
        self.editor.image_editor.drop_selection(sprite);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, editor: &Editor, sprite: &Sprite, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, sprite.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", sprite.asset.id)).frame(bottom_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty = if base.is_dirty() { " (modified)" } else { "" };
                let frames_plural = if sprite.num_frames > 1 { "s" } else { "" };
                ui.add(egui::Label::new(format!(
                    "{} bytes [{}x{}, {} frame{}]{}",
                    sprite.data_size(),
                    sprite.width,
                    sprite.height,
                    sprite.num_frames,
                    frames_plural, dirty
                )).truncate());

                if let Some(sprite) = editor.image_picker.get_selected_image() {
                    ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let spacing = ui.spacing().item_spacing;
                            ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                            ui.add_space(1.0);
                            let (hover_x, hover_y) = (editor.image_editor.hover_pos.x, editor.image_editor.hover_pos.y);
                            ui.label(format!("({}, {}) sprite {}", hover_x, hover_y, sprite));
                            ui.spacing_mut().item_spacing = spacing;
                        });
                    });
                }
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        self.dialogs.show(wc, &mut self.editor, sprite);

        let (min_size, default_size) = AssetEditorBase::calc_image_editor_window_size(sprite);
        self.base.show_window(wc, sprite, min_size, default_size, |ui, wc, sprite, base| {
            Self::show_footer(ui, wc, &self.editor, sprite, base);
            self.editor.show(ui, wc, &mut self.dialogs, sprite);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    add_frames_dialog: AddFramesDialog,
    rm_frames_dialog: RemoveFramesDialog,
    import_dialog: ImportDialog,
    export_dialog: ExportDialog,
    create_colorset_dialog: CreateColorsetDialog,
}

impl Dialogs {
    fn new(id: DataAssetId) -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_frames_dialog: AddFramesDialog::new(),
            rm_frames_dialog: RemoveFramesDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
            create_colorset_dialog: CreateColorsetDialog::new(id),
        }
    }

    fn ensure_valid_selected_image(editor: &mut Editor, sprite: &Sprite, set_undo_target: bool) {
        if editor.image_editor.get_selected_image() >= sprite.num_frames {
            let selected_image = sprite.num_frames - 1;
            editor.image_picker.set_selected_image(Some(selected_image));
            let no_selection_change = ! editor.image_editor.set_selected_image(selected_image, sprite);
            if no_selection_change && set_undo_target {
                editor.image_editor.set_undo_target(sprite);
            }
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, sprite: &mut Sprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            Self::ensure_valid_selected_image(editor, sprite, false);
        }
        if self.add_frames_dialog.open && self.add_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.image_editor.set_undo_target(sprite);
        }
        if self.rm_frames_dialog.open && self.rm_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            Self::ensure_valid_selected_image(editor, sprite, false);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, sprite);
        }
        if self.import_dialog.open && self.import_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            Self::ensure_valid_selected_image(editor, sprite, true);
        }
        if self.create_colorset_dialog.open && self.create_colorset_dialog.show(wc, sprite) {
            editor.color_picker.set_colorset(self.create_colorset_dialog.created_colorset_index);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    import_frame_sys_dlg_id: String,
    color_picker: ColorPickerWidget,
    image_picker: ImagePickerWidget,
    image_editor: ImageEditorWidget<Sprite>,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            import_frame_sys_dlg_id: format!("editor_{}_import_frame", asset_id),
            color_picker: ColorPickerWidget::new(format!("editor_{}_color_picker", asset_id), colors::RED, colors::GREEN, true),
            image_picker: ImagePickerWidget::new(),
            image_editor: ImageEditorWidget::<Sprite>::new(),
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sprite: &mut Sprite) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&self.import_frame_sys_dlg_id) {
            let image = match ImagePixels::load_png(&filename) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", filename.display(), e));
                    return;
                }
            };
            self.image_editor.paste_pixels(sprite, image);
        }

        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Sprite", |ui| {
                    if ui.add(menu_item(IMAGES.import, " Import...")).clicked() {
                        dialogs.import_dialog.set_open(wc, sprite);
                    }
                    if ui.add(menu_item(IMAGES.export, " Export...")).clicked() {
                        dialogs.export_dialog.set_open(wc, sprite);
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, sprite, self.color_picker.state.right_color);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(self.image_editor.can_undo(), menu_item(IMAGES.undo, " Undo")).clicked() {
                        self.image_editor.undo(sprite);
                    }

                    ui.separator();

                    let has_selection = ! self.image_editor.selection.is_empty();
                    if ui.add_enabled(has_selection, menu_item(IMAGES.cut, " Cut")).clicked() {
                        self.image_editor.cut(wc, sprite, self.color_picker.state.right_color);
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.copy, " Copy")).clicked() {
                        self.image_editor.copy(wc, sprite);
                    }
                    if ui.add_enabled(wc.image_clipboard.is_some(), menu_item(IMAGES.paste, " Paste")).clicked() {
                        self.image_editor.paste(wc, sprite);
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.trash, " Delete selection")).clicked() {
                        self.image_editor.delete_selection(sprite, self.color_picker.state.right_color);
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.import, " Paste from file...")).clicked() {
                        wc.sys_dialogs.open_file(
                            Some(wc.egui.window),
                            self.import_frame_sys_dlg_id.clone(),
                            "sprite",
                            "Paste From File",
                            &[
                                ("PNG files (*.png)", &["png"]),
                                ("All files (*.*)", &["*"]),
                            ]
                        );
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.add, " Insert frames...")).clicked() {
                        dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Insert, self.image_editor.get_selected_image(),
                                                           self.color_picker.state.right_color);
                    }
                    if ui.add(menu_item(IMAGES.add, " Append frames...")).clicked() {
                        dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Append, self.image_editor.get_selected_image(),
                                                           self.color_picker.state.right_color);
                    }
                    if ui.add(menu_item(IMAGES.trash, " Remove frames...")).clicked() {
                        dialogs.rm_frames_dialog.set_open(wc, sprite, self.image_editor.get_selected_image());
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, sprite: &mut Sprite) {
        egui::Panel::top(format!("editor_panel_{}_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Tool:");
                ui.add_space(1.0);
                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.image_editor.get_tool() == ImageDrawingTool::Pencil)
                          .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Pencil)).on_hover_text("Pencil").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Pencil);
                }

                if ui.add(egui::Button::image(IMAGES.fill)
                          .selected(self.image_editor.get_tool() == ImageDrawingTool::Fill)
                          .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Fill)).on_hover_text("Fill").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Fill);
                }

                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.image_editor.get_tool() == ImageDrawingTool::Select)
                          .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Select)).on_hover_text("Select").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Select);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(egui::Button::image(IMAGES.v_flip)).on_hover_text("Flip vertically").clicked() {
                    self.image_editor.vflip(sprite, self.color_picker.state.right_color);
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Flip horizontally").clicked() {
                    self.image_editor.hflip(sprite, self.color_picker.state.right_color);
                }
                ui.add_space(5.0);
                if ui.add(egui::Button::image(IMAGES.rot_cw)).on_hover_text("Rotate 90° clockwise").clicked() {
                    self.image_editor.rotate(sprite, ImageRotation::CW90, self.color_picker.state.right_color);
                }
                if ui.add(egui::Button::image(IMAGES.rot_ccw)).on_hover_text("Rotate 90° counter-clockwise").clicked() {
                    self.image_editor.rotate(sprite, ImageRotation::CCW90, self.color_picker.state.right_color);
                }
                ui.spacing_mut().item_spacing = spacing;

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.grid)
                                  .selected(self.image_editor.display.has_bits(ImageDisplay::GRID))
                                  .frame_when_inactive(self.image_editor.display.has_bits(ImageDisplay::GRID)))
                            .on_hover_text("Grid").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::GRID);
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                                  .selected(self.image_editor.display.is_transparent())
                                  .frame_when_inactive(self.image_editor.display.is_transparent()))
                            .on_hover_text("Transparency").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::TRANSPARENT);
                            }
                        ui.add_space(1.0);
                        ui.label("Display:");

                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let mut cur_zoom_option = ImageZoomOption::from_image_editor_zoom(self.image_editor.zoom);
                        egui::ComboBox::from_id_salt(format!("sprite_editor_{}_zoom_combo", self.asset_id))
                            .selected_text(cur_zoom_option.name())
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for option in IMAGE_ZOOM_OPTIONS {
                                    if option.is_custom() && ! cur_zoom_option.is_custom() { continue; }
                                    ui.selectable_value(&mut cur_zoom_option, option, option.name());
                                }
                            });
                        self.image_editor.zoom = cur_zoom_option.image_editor_zoom(self.image_editor.zoom);
                        ui.add_space(1.0);
                        ui.label("Zoom:");

                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    pub fn reload_images(wc: &mut WindowContext, asset: &impl ImageCollection) {
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque, true);
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, true);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sprite: &mut Sprite) {
        self.show_menu_bar(ui, wc, dialogs, sprite);
        self.show_toolbar(ui, wc, sprite);

        // item picker:
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            self.image_picker.zoom = 80.0 / sprite.width as f32;
            self.image_picker.display = self.image_editor.display;
            let slot = sprite.texture_slot(self.image_picker.display.is_transparent(), false);
            let texture = sprite.texture(wc.tex_man, wc.egui.ctx, slot);
            self.image_picker.show(ui, wc.settings, sprite, texture, wc.settings.image_bg_color);
            if let Some(selected_image) = self.image_picker.get_selected_image() {
                self.image_editor.set_selected_image(selected_image, sprite);
            }
        });

        // color picker:
        egui::Panel::right(format!("editor_panel_{}_right", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            match self.color_picker.show(ui, wc) {
                ColorPickerResponse::None => {}
                ColorPickerResponse::CreateColorset => {
                    dialogs.create_colorset_dialog.set_open(wc, self.image_editor.get_selected_image());
                }
            }
        });

        // image:
        egui::CentralPanel::default().show(ui, |ui| {
            let colors = (self.color_picker.state.left_color, self.color_picker.state.right_color);
            self.image_editor.show(ui, wc, sprite, colors);
            self.color_picker.maybe_set_colors(
                self.image_editor.pick_left_color.take(),
                self.image_editor.pick_right_color.take()
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            self.image_editor.handle_keyboard(ui, wc, sprite, self.color_picker.state.right_color);
        }
    }
}
