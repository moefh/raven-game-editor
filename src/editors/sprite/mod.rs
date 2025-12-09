mod properties;
mod add_frames;
mod remove_frames;
mod import;
mod export;

use crate::misc::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollection, ImagePixels, TextureSlot};
use crate::data_asset::{DataAssetId, Sprite, GenericAsset};

use properties::PropertiesDialog;
use remove_frames::RemoveFramesDialog;
use add_frames::{AddFramesDialog, AddFramesAction};
use import::ImportDialog;
use export::ExportDialog;
use super::DataAssetEditor;
use super::widgets::{ColorPickerWidget, ImagePickerWidget, ImageEditorWidget, ImageDrawingTool, ImageDisplay};

pub struct SpriteEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, sprite: &mut Sprite) {
        self.editor.image_editor.drop_selection(sprite);
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        self.dialogs.show(wc, &mut self.editor, sprite);

        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        let (min_size, default_size) = DataAssetEditor::calc_image_editor_window_size(sprite);
        window.min_size(min_size).default_size(default_size).show(wc.egui.ctx, |ui| {
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
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_frames_dialog: AddFramesDialog::new(),
            rm_frames_dialog: RemoveFramesDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, sprite: &mut Sprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(sprite.num_frames-1);
            editor.image_editor.set_selected_image(editor.image_picker.selected_image, sprite);
            editor.image_editor.set_undo_target(sprite);
        }
        if self.add_frames_dialog.open && self.add_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.image_editor.set_undo_target(sprite);
        }
        if self.rm_frames_dialog.open && self.rm_frames_dialog.show(wc, sprite) {
            Editor::reload_images(wc, sprite);
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(sprite.num_frames-1);
            editor.image_editor.set_selected_image(editor.image_picker.selected_image, sprite);
            editor.image_editor.set_undo_target(sprite);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, sprite);
        }
        if self.import_dialog.open && self.import_dialog.show(wc, sprite) {
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(sprite.num_frames-1);
            if ! editor.image_editor.set_selected_image(editor.image_picker.selected_image, sprite) {
                editor.image_editor.set_undo_target(sprite);
            }
            editor.image_editor.set_image_changed();
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    color_picker: ColorPickerWidget,
    image_picker: ImagePickerWidget,
    image_editor: ImageEditorWidget,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            color_picker: ColorPickerWidget::new(0b000011, 0b001100),
            image_picker: ImagePickerWidget::new(),
            image_editor: ImageEditorWidget::new(),
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, sprite: &mut Sprite) {
        let import_frame_dlg_id = format!("editor_{}_import_frame", sprite.asset.id);
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&import_frame_dlg_id) {
            let image = match ImagePixels::load_png(&filename) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", filename.display(), e));
                    return;
                }
            };
            self.image_editor.paste_pixels(sprite, image);
        }

        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Sprite", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Import...").clicked() {
                            dialogs.import_dialog.set_open(wc, sprite);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            dialogs.export_dialog.set_open(wc, sprite);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, sprite, self.color_picker.right_color);
                        }
                    });
                });
                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.undo).max_width(14.0).max_height(14.0));
                        if ui.button("Undo").clicked() {
                            self.image_editor.undo(sprite);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.cut).max_width(14.0).max_height(14.0));
                        if ui.button("Cut").clicked() {
                            self.image_editor.cut(wc, sprite, self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.copy).max_width(14.0).max_height(14.0));
                        if ui.button("Copy").clicked() {
                            self.image_editor.copy(wc, sprite);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.paste).max_width(14.0).max_height(14.0));
                        if ui.button("Paste").clicked() {
                            self.image_editor.paste(wc, sprite);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Delete selection").clicked() {
                            self.image_editor.delete_selection(sprite, self.color_picker.right_color);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Paste from file...").clicked() {
                            wc.sys_dialogs.open_file(
                                Some(wc.egui.window),
                                import_frame_dlg_id,
                                "Paste From File",
                                &[
                                    ("PNG files (*.png)", &["png"]),
                                    ("All files (*.*)", &["*"]),
                                ]
                            );
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.add).max_width(14.0).max_height(14.0));
                        if ui.button("Insert frames...").clicked() {
                            dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Insert, self.image_picker.selected_image,
                                                               self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.add).max_width(14.0).max_height(14.0));
                        if ui.button("Append frames...").clicked() {
                            dialogs.add_frames_dialog.set_open(wc, AddFramesAction::Append, self.image_picker.selected_image,
                                                               self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Remove frames...").clicked() {
                            dialogs.rm_frames_dialog.set_open(wc, sprite, self.image_picker.selected_image);
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, sprite: &mut Sprite) {
        egui::TopBottomPanel::top(format!("editor_panel_{}_toolbar", self.asset_id)).show_inside(ui, |ui| {
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

                if ui.add(egui::Button::image(IMAGES.v_flip)).on_hover_text("Vertical Flip").clicked() {
                    self.image_editor.vflip(sprite, self.color_picker.right_color);
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Horizontal Flip").clicked() {
                    self.image_editor.hflip(sprite, self.color_picker.right_color);
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

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes [{} frames]", sprite.data_size(), sprite.num_frames));
        });

        // item picker:
        egui::SidePanel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.image_picker.zoom = 80.0 / sprite.width as f32;
            self.image_picker.display = self.image_editor.display;
            let texture = sprite.texture(wc.tex_man, wc.egui.ctx, self.image_picker.display.texture_slot());
            self.image_picker.show(ui, wc.settings, sprite, texture);
            self.image_editor.set_selected_image(self.image_picker.selected_image, sprite);
        });

        // color picker:
        egui::SidePanel::right(format!("editor_panel_{}_right", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.color_picker.show(ui, wc);
        });

        // image:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let colors = (self.color_picker.left_color, self.color_picker.right_color);
            self.image_editor.show(ui, wc, sprite, colors);
            self.color_picker.maybe_set_colors(
                self.image_editor.pick_left_color.take(),
                self.image_editor.pick_right_color.take()
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            self.image_editor.handle_keyboard(ui, wc, sprite, self.color_picker.right_color);
        }
    }
}
