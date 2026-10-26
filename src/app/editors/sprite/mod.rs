mod properties;
mod add_frames;
mod add_imported_frames;
mod remove_frames;
mod import;
mod export;
mod export_selected_frames;
mod manage_frame_collections;

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
    SpriteFrameFixer,
};
use super::dialogs::CreateColorsetDialog;
use super::widgets::{
    ColorPickerWidget,
    ColorPickerResponse,
    ImagePickerWidget,
    ImagePickerSelection,
    ImageEditorWidget,
    ImageDrawingTool,
    ImageDisplay,
};
use super::super::{
    menu_item,
    menu_item_no_image,
};

use properties::PropertiesDialog;
use remove_frames::RemoveFramesDialog;
use add_frames::AddFramesDialog;
use add_imported_frames::AddImportedFramesDialog;
use import::ImportDialog;
use export::ExportDialog;
use export_selected_frames::ExportSelectedFramesDialog;
use manage_frame_collections::ManageFrameCollectionsDialog;

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
                    "{} bytes [{}x{}, {} frame{}, {}mirrored]{}",
                    sprite.data_size(),
                    sprite.width,
                    sprite.height,
                    sprite.num_frames,
                    frames_plural,
                    if sprite.add_mirrors { "" } else { "not " },
                    dirty
                )).truncate());

                if let ImagePickerSelection::Some(sprite) = editor.image_picker.get_selected_image_l() {
                    ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let spacing = ui.spacing().item_spacing;
                            ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                            ui.add_space(1.0);
                            ui.label(format!("[sprite {}]", sprite));

                            if let Some(hover_pos) = editor.get_hover_pos() {
                                ui.add_space(12.0);
                                ui.label(format!("({},{})", hover_pos.x, hover_pos.y));
                            }

                            if let Some(sel_rect) = editor.get_selection_rectangle() && sel_rect.is_positive() {
                                ui.add_space(12.0);
                                ui.label(format!("[sel {}x{}]", sel_rect.width(), sel_rect.height()));
                            }

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
        let min_size = min_size.max(egui::Vec2::new(580.0, 300.0));
        let default_size = default_size.max(egui::Vec2::new(580.0, 300.0));
        self.base.show_window(wc, sprite, min_size, default_size, |ui, wc, sprite, base| {
            Self::show_footer(ui, wc, &self.editor, sprite, base);
            self.editor.show(ui, wc, &mut self.dialogs, sprite);
        });
    }
}

impl SpriteFrameFixer for SpriteEditor {
    fn move_frame(&mut self, src_index: u32, dest_index: u32) {
        self.editor.image_editor.move_frame_undo_history(src_index, dest_index);
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    add_frames_dialog: AddFramesDialog,
    add_imported_frames_dialog: AddImportedFramesDialog,
    rm_frames_dialog: RemoveFramesDialog,
    import_dialog: ImportDialog,
    export_dialog: ExportDialog,
    export_selected_frames_dialog: ExportSelectedFramesDialog,
    manage_frame_collections_dialog: ManageFrameCollectionsDialog,
    create_colorset_dialog: CreateColorsetDialog,
}

impl Dialogs {
    fn new(id: DataAssetId) -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_frames_dialog: AddFramesDialog::new(),
            add_imported_frames_dialog: AddImportedFramesDialog::new(),
            rm_frames_dialog: RemoveFramesDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
            export_selected_frames_dialog: ExportSelectedFramesDialog::new(),
            manage_frame_collections_dialog: ManageFrameCollectionsDialog::new(),
            create_colorset_dialog: CreateColorsetDialog::new(id),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, sprite: &mut Sprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.ensure_valid_selected_image(wc, sprite);
        }
        if self.add_frames_dialog.open && self.add_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
        }
        if self.add_imported_frames_dialog.open && self.add_imported_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
        }
        if self.rm_frames_dialog.open && self.rm_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.ensure_valid_selected_image(wc, sprite);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, sprite);
        }
        if self.export_selected_frames_dialog.open {
            self.export_selected_frames_dialog.show(wc, sprite);
        }
        if self.import_dialog.open && self.import_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.ensure_valid_selected_image(wc, sprite);
            editor.image_editor.clear_undo_history();
        }
        if self.manage_frame_collections_dialog.open && self.manage_frame_collections_dialog.show(wc, sprite) {
            editor.set_sel_frame_collection(wc, editor.sel_frame_collection, sprite);
        }
        if self.create_colorset_dialog.open && self.create_colorset_dialog.show(wc, sprite) {
            editor.color_picker.set_colorset(self.create_colorset_dialog.created_colorset_index);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    import_frame_sys_dlg_id: String,
    sel_frame_collection: Option<usize>,
    frame_collection_indices: Vec<u32>,
    color_picker: ColorPickerWidget,
    image_picker: ImagePickerWidget,
    image_editor: ImageEditorWidget<Sprite>,
}

impl Editor {
    const ALL_FRAMES_COLLECTION_NAME: &str = "<all>";

    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            import_frame_sys_dlg_id: format!("editor_{}_import_frame", asset_id),
            sel_frame_collection: None,
            frame_collection_indices: Vec::new(),
            color_picker: ColorPickerWidget::new(format!("editor_{}_color_picker", asset_id), colors::RED, colors::GREEN, true),
            image_picker: ImagePickerWidget::new(),
            image_editor: ImageEditorWidget::new(),
        }
    }

    fn get_selection_rectangle(&self) -> Option<egui::Rect> {
        self.image_editor.selection.get_rect()
    }

    fn get_hover_pos(&self) -> Option<egui::Vec2> {
        Some(self.image_editor.hover_pos)
    }

    fn ensure_valid_selected_image(&mut self, wc: &WindowContext, sprite: &Sprite) {
        if let Some(col_index) = self.sel_frame_collection &&
            let Some(md) = wc.store_metadata.get_sprite(&self.asset_id) &&
            let Some(col) = md.frames.get(col_index) &&
            let Some(first_frame) = col.indices.first() {
                if ! col.indices.contains(&self.image_editor.get_selected_image()) {
                    self.image_picker.set_selected_image_l(ImagePickerSelection::Some(*first_frame));
                }
            } else {
                if self.image_editor.get_selected_image() >= sprite.num_frames {
                    let selected_image = sprite.num_frames - 1;
                    self.image_picker.set_selected_image_l(ImagePickerSelection::Some(selected_image));
                }
            }
    }

    fn set_sel_frame_collection(&mut self, wc: &WindowContext, sel_frame_collection: Option<usize>, sprite: &Sprite) {
        self.sel_frame_collection = sel_frame_collection;
        if let Some(col_index) = self.sel_frame_collection &&
            let Some(md) = wc.store_metadata.get_sprite(&self.asset_id) &&
            let Some(col) = md.frames.get(col_index) &&
            ! col.indices.is_empty() {
                self.frame_collection_indices.splice(.., col.indices.clone());
                self.ensure_valid_selected_image(wc, sprite);
            } else {
                self.frame_collection_indices.clear();
            }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sprite: &mut Sprite) {
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_frame_sys_dlg_id) {
            let image = match file.read_data().and_then(|data| ImagePixels::load_png(&data)) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", file.filename(), e));
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
                    if ui.add_enabled(self.image_editor.can_redo(), menu_item(IMAGES.redo, " Redo")).clicked() {
                        self.image_editor.redo(sprite);
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

                    if ui.add(menu_item_no_image(" Add frames from file...")).clicked() {
                        dialogs.add_imported_frames_dialog.set_open(
                            wc,
                            self.image_editor.get_selected_image(),
                            self.color_picker.state.right_color,
                            sprite,
                        );
                    }

                    if ui.add(menu_item(IMAGES.export, " Export frames to file...")).clicked() {
                        dialogs.export_selected_frames_dialog.set_open(
                            wc,
                            self.image_editor.get_selected_image(),
                            sprite,
                        );
                    }

                    ui.separator();

                    if ui.add(menu_item(IMAGES.add, " Add frames...")).clicked() {
                        dialogs.add_frames_dialog.set_open(
                            wc,
                            self.image_editor.get_selected_image(),
                            self.color_picker.state.right_color
                        );
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
                if ui.add(
                    egui::Button::image(IMAGES.pen)
                        .selected(self.image_editor.get_tool() == ImageDrawingTool::Pencil)
                        .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Pencil)
                ).on_hover_text("Pencil (A)").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Pencil);
                }

                if ui.add(
                    egui::Button::image(IMAGES.fill)
                        .selected(self.image_editor.get_tool() == ImageDrawingTool::Fill)
                        .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Fill)
                ).on_hover_text("Fill (S)").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Fill);
                }

                if ui.add(
                    egui::Button::image(IMAGES.select)
                        .selected(self.image_editor.get_tool() == ImageDrawingTool::Select)
                        .frame_when_inactive(self.image_editor.get_tool() == ImageDrawingTool::Select)
                ).on_hover_text("Select (D)").clicked() {
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
                            .on_hover_text("Toggle Grid Display").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::GRID);
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                                  .selected(self.image_editor.display.is_transparent())
                                  .frame_when_inactive(self.image_editor.display.is_transparent()))
                            .on_hover_text("Toggle Transparency Display").clicked() {
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

    fn show_frame_collection_combo(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        sprite: &Sprite
    ) {
        let sprite_md = wc.store_metadata.get_sprite(&self.asset_id);
        let selected_frame_collection_name = if let Some(md) = sprite_md && let Some(col_index) = self.sel_frame_collection {
            if let Some(col) = md.frames.get(col_index) {
                col.name.as_ref()
            } else {
                self.sel_frame_collection = None;
                Self::ALL_FRAMES_COLLECTION_NAME
            }
        } else {
            Self::ALL_FRAMES_COLLECTION_NAME
        };

        let mut sel_frame_collection = self.sel_frame_collection;
        egui::ComboBox::from_id_salt(format!("sprite_editor_{}_frame_collection_combo", self.asset_id))
            .selected_text(selected_frame_collection_name)
            .width(60.0)
            .truncate()
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut sel_frame_collection, None, Self::ALL_FRAMES_COLLECTION_NAME);
                if let Some(collections) = wc.store_metadata.get_sprite(&self.asset_id) {
                    for (index, frames) in collections.frames.iter().enumerate() {
                        ui.selectable_value(&mut sel_frame_collection, Some(index), &frames.name);
                    }
                }
                ui.separator();
                ui.selectable_value(&mut sel_frame_collection, Some(usize::MAX), "Manage...");
            });
        if sel_frame_collection.is_some_and(|c| c == usize::MAX) {
            dialogs.manage_frame_collections_dialog.set_open(wc, sprite);
        } else if sel_frame_collection != self.sel_frame_collection {
            self.set_sel_frame_collection(wc, sel_frame_collection, sprite);
        }
    }

    pub fn reload_images(wc: &mut WindowContext, asset: &impl ImageCollection) {
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque, true);
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, true);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sprite: &mut Sprite) {
        self.show_menu_bar(ui, wc, dialogs, sprite);
        self.show_toolbar(ui, wc, sprite);

        // frame picker:
        egui::Panel::left(format!("editor_panel_{}_frame_picker", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            self.show_frame_collection_combo(ui, wc, dialogs, sprite);
            ui.add_space(2.0);
            self.image_picker.zoom = Some(80.0 / sprite.width as f32);
            self.image_picker.display = self.image_editor.display;
            let image_indices = if self.frame_collection_indices.is_empty() { None } else {
                Some(self.frame_collection_indices.as_ref())
            };
            self.image_picker.show(ui, wc, sprite, image_indices);
            if let ImagePickerSelection::Some(selected_image) = self.image_picker.get_selected_image_l() {
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
