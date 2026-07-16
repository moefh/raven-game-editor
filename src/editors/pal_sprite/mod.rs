mod properties;
mod add_frames;
mod remove_frames;
mod edit_palette;
mod import;
mod export;

use crate::misc::IMAGES;
use crate::app::{
    menu_item,
    menu_item_no_image,
    WindowContext,
    SysDialogResponse,
};
use crate::image::{ImageCollection, ImagePixels, TextureSlot, ImageRotation};
use crate::data_asset::{DataAssetId, PalSprite, GenericAsset};

use properties::PropertiesDialog;
use remove_frames::RemoveFramesDialog;
use add_frames::{AddFramesDialog, AddFramesAction};
use edit_palette::EditPaletteDialog;
use import::ImportDialog;
use export::ExportDialog;
use super::{
    IMAGE_ZOOM_OPTIONS,
    ImageZoomOption,
    AssetEditorBase,
};
use super::widgets::{
    PalColorPickerWidget,
    PalColorPickerAction,
    ImagePickerWidget,
    ImageEditorWidget,
    ImageEditorAction,
    ImageDrawingTool,
    ImageDisplay,
};

pub struct PalSpriteEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl PalSpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PalSpriteEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, pal_sprite: &mut PalSprite) {
        self.editor.image_editor.drop_selection(pal_sprite);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, editor: &Editor, pal_sprite: &PalSprite, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, pal_sprite.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", pal_sprite.asset.id)).frame(bottom_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty = if base.is_dirty() { " (modified)" } else { "" };
                let frames_plural = if pal_sprite.num_frames > 1 { "s" } else { "" };
                ui.add(egui::Label::new(format!(
                    "{} bytes [{}x{}, {} bpp, {} frame{}]{}",
                    pal_sprite.data_size(),
                    pal_sprite.width,
                    pal_sprite.height,
                    pal_sprite.depth.bits_per_pixel(),
                    pal_sprite.num_frames,
                    frames_plural,
                    dirty
                )).truncate());

                if let Some(sprite) = editor.image_picker.get_selected_image() {
                    ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let spacing = ui.spacing().item_spacing;
                            ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                            ui.add_space(1.0);
                            ui.label(format!("sprite {}", sprite));
                            ui.spacing_mut().item_spacing = spacing;
                        });
                    });
                }
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) {
        self.dialogs.show(wc, &mut self.editor, pal_sprite);

        let (min_size, default_size) = AssetEditorBase::calc_image_editor_window_size(pal_sprite);
        self.base.show_window(wc, pal_sprite, min_size, default_size, |ui, wc, pal_sprite, base| {
            Self::show_footer(ui, wc, &self.editor, pal_sprite, base);
            self.editor.show(ui, wc, &mut self.dialogs, pal_sprite);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    add_frames_dialog: AddFramesDialog,
    rm_frames_dialog: RemoveFramesDialog,
    edit_pal_dialog: EditPaletteDialog,
    import_dialog: ImportDialog,
    export_dialog: ExportDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_frames_dialog: AddFramesDialog::new(),
            rm_frames_dialog: RemoveFramesDialog::new(),
            edit_pal_dialog: EditPaletteDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
        }
    }

    fn ensure_valid_selected_image(editor: &mut Editor, pal_sprite: &PalSprite, set_undo_target: bool) {
        if editor.image_editor.get_selected_image() >= pal_sprite.num_frames {
            let selected_image = pal_sprite.num_frames - 1;
            editor.image_picker.set_selected_image(Some(selected_image));
            let no_selection_change = ! editor.image_editor.set_selected_image(selected_image, pal_sprite);
            if no_selection_change && set_undo_target {
                editor.image_editor.set_undo_target(pal_sprite);
            }
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, pal_sprite: &mut PalSprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, pal_sprite) {
            Editor::reload_images(wc, pal_sprite);
            Self::ensure_valid_selected_image(editor, pal_sprite, false);
        }
        if self.add_frames_dialog.open && self.add_frames_dialog.show(wc, pal_sprite) {
            Editor::reload_images(wc, pal_sprite);
            editor.image_editor.set_undo_target(pal_sprite);
        }
        if self.rm_frames_dialog.open && self.rm_frames_dialog.show(wc, pal_sprite) {
            Editor::reload_images(wc, pal_sprite);
            Self::ensure_valid_selected_image(editor, pal_sprite, false);
        }
        if self.edit_pal_dialog.open && self.edit_pal_dialog.show(wc, pal_sprite) {
            Editor::reload_images(wc, pal_sprite);
            editor.image_editor.set_undo_target(pal_sprite);
            Self::ensure_valid_selected_image(editor, pal_sprite, false);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, pal_sprite);
        }
        if self.import_dialog.open && self.import_dialog.show(wc, pal_sprite) {
            Editor::reload_images(wc, pal_sprite);
            Self::ensure_valid_selected_image(editor, pal_sprite, true);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    import_frame_dlg_id: String,
    color_picker: PalColorPickerWidget,
    image_picker: ImagePickerWidget,
    image_editor: ImageEditorWidget<PalSprite>,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            import_frame_dlg_id: format!("editor_{}_import_frame", asset_id),
            color_picker: PalColorPickerWidget::new(0, 1),
            image_picker: ImagePickerWidget::new(),
            image_editor: ImageEditorWidget::<PalSprite>::new().with_image_display(ImageDisplay::grid_only()),
        }
    }

    fn get_pal_color(color_index: u8, pal_sprite: &PalSprite) -> u8 {
        let color_index = color_index as usize;
        pal_sprite.palette[color_index % pal_sprite.palette.len()]
    }

    fn get_right_color(&self, pal_sprite: &PalSprite) -> u8 {
        Self::get_pal_color(self.color_picker.state.right_index, pal_sprite)
    }

    fn get_selected_colors(&self, pal_sprite: &PalSprite) -> (u8, u8) {
        (
            Self::get_pal_color(self.color_picker.state.left_index, pal_sprite),
            Self::get_pal_color(self.color_picker.state.right_index, pal_sprite)
        )
    }

    fn force_palette(&mut self, pal_sprite: &mut PalSprite) {
        if pal_sprite.force_palette() {
            self.image_editor.set_image_changed();
        }
        self.image_editor.force_palette(&pal_sprite.palette, &pal_sprite.color_to_palette_index_map);
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, pal_sprite: &mut PalSprite) {
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&self.import_frame_dlg_id) {
            let image = match ImagePixels::load_png(&filename) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", filename.display(), e));
                    return;
                }
            };
            self.image_editor.paste_pixels(pal_sprite, image);
            self.image_editor.force_palette(&pal_sprite.palette, &pal_sprite.color_to_palette_index_map);
        }

        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Paletted Sprite", |ui| {
                    if ui.add(menu_item(IMAGES.import, " Import...")).clicked() {
                        dialogs.import_dialog.set_open(wc, pal_sprite);
                    }
                    if ui.add(menu_item(IMAGES.export, " Export...")).clicked() {
                        dialogs.export_dialog.set_open(wc, pal_sprite);
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, pal_sprite, self.get_right_color(pal_sprite));
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(self.image_editor.can_undo(), menu_item(IMAGES.undo, " Undo")).clicked() {
                        self.image_editor.undo(pal_sprite);
                    }

                    ui.separator();

                    let has_selection = ! self.image_editor.selection.is_empty();
                    if ui.add_enabled(has_selection, menu_item(IMAGES.cut, " Cut")).clicked() {
                        self.image_editor.cut(wc, pal_sprite, self.get_right_color(pal_sprite));
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.copy, " Copy")).clicked() {
                        self.image_editor.copy(wc, pal_sprite);
                    }
                    if ui.add_enabled(wc.image_clipboard.is_some(), menu_item(IMAGES.paste, " Paste")).clicked() {
                        self.image_editor.paste(wc, pal_sprite);
                        self.force_palette(pal_sprite);
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.trash, " Delete selection")).clicked() {
                        self.image_editor.delete_selection(pal_sprite, self.get_right_color(pal_sprite));
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.import, " Paste from file...")).clicked() {
                        wc.sys_dialogs.open_file(
                            Some(wc.egui.window),
                            self.import_frame_dlg_id.clone(),
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
                        dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Insert,
                            self.image_editor.get_selected_image(),
                            self.get_right_color(pal_sprite));
                    }
                    if ui.add(menu_item(IMAGES.add, " Append frames...")).clicked() {
                        dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Append,
                            self.image_editor.get_selected_image(),
                            self.get_right_color(pal_sprite));
                    }
                    if ui.add_enabled(pal_sprite.num_frames > 1, menu_item(IMAGES.trash, " Remove frames...")).clicked() {
                        dialogs.rm_frames_dialog.set_open(wc, pal_sprite, self.image_editor.get_selected_image());
                    }

                    ui.separator();

                    if ui.add(menu_item_no_image(" Edit palette...")).clicked() {
                        dialogs.edit_pal_dialog.set_open(wc, pal_sprite);
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, pal_sprite: &mut PalSprite) {
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
                    self.image_editor.vflip(pal_sprite, self.get_right_color(pal_sprite));
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Flip horizontally").clicked() {
                    self.image_editor.hflip(pal_sprite, self.get_right_color(pal_sprite));
                }
                ui.add_space(5.0);
                if ui.add(egui::Button::image(IMAGES.rot_cw)).on_hover_text("Rotate 90° clockwise").clicked() {
                    self.image_editor.rotate(pal_sprite, ImageRotation::CW90, self.get_right_color(pal_sprite));
                }
                if ui.add(egui::Button::image(IMAGES.rot_ccw)).on_hover_text("Rotate 90° counter-clockwise").clicked() {
                    self.image_editor.rotate(pal_sprite, ImageRotation::CCW90, self.get_right_color(pal_sprite));
                }

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.grid)
                                  .selected(self.image_editor.display.has_bits(ImageDisplay::GRID))
                                  .frame_when_inactive(self.image_editor.display.has_bits(ImageDisplay::GRID)))
                            .on_hover_text("Grid").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::GRID);
                            }
                        ui.add_space(1.0);
                        ui.label("Display:");

                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let mut cur_zoom_option = ImageZoomOption::from_image_editor_zoom(self.image_editor.zoom);
                        egui::ComboBox::from_id_salt(format!("pal_sprite_editor_{}_zoom_combo", self.asset_id))
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
                    });
                });

                ui.spacing_mut().item_spacing = spacing;
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    pub fn reload_images(wc: &mut WindowContext, asset: &impl ImageCollection) {
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque, true);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, pal_sprite: &mut PalSprite) {
        self.show_menu_bar(ui, wc, dialogs, pal_sprite);
        self.show_toolbar(ui, wc, pal_sprite);

        // item picker:
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            self.image_picker.zoom = 80.0 / pal_sprite.width as f32;
            self.image_picker.display = self.image_editor.display;
            let slot = pal_sprite.texture_slot(self.image_picker.display.is_transparent(), false);
            let texture = pal_sprite.texture(wc.tex_man, wc.egui.ctx, slot);
            self.image_picker.show(ui, wc.settings, pal_sprite, texture, wc.settings.image_bg_color);
            if let Some(selected_image) = self.image_picker.get_selected_image() {
                self.image_editor.set_selected_image(selected_image, pal_sprite);
            }
        });

        // color picker:
        egui::Panel::right(format!("editor_panel_{}_right", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            let action = self.color_picker.show(ui, wc, &pal_sprite.palette[0..pal_sprite.depth.num_colors() as usize]);
            if matches!(action, PalColorPickerAction::EditPalette) {
                dialogs.edit_pal_dialog.set_open(wc, pal_sprite);
            }
        });

        // image:
        egui::CentralPanel::default().show(ui, |ui| {
            let colors = self.get_selected_colors(pal_sprite);
            self.image_editor.show(ui, wc, pal_sprite, colors);
            self.color_picker.maybe_set_colors(
                self.image_editor.pick_left_color.take(),
                self.image_editor.pick_right_color.take(),
                &pal_sprite.color_to_palette_index_map,
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            let action = self.image_editor.handle_keyboard(ui, wc, pal_sprite, self.get_right_color(pal_sprite));
            if matches!(action, ImageEditorAction::Paste) {
                self.force_palette(pal_sprite);
            }
        }
    }
}
