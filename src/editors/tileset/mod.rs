mod properties;
mod add_tiles;
mod remove_tiles;
mod import;
mod export;

use crate::misc::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{ImageCollection, ImagePixels, TextureSlot};
use crate::data_asset::{Tileset, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use remove_tiles::RemoveTilesDialog;
use add_tiles::{AddTilesDialog, AddTilesAction};
use export::ExportDialog;
use import::ImportDialog;
use super::DataAssetEditor;
use super::widgets::{ColorPickerWidget, ImagePickerWidget, ImageEditorWidget, ImageDrawingTool, ImageDisplay};

pub struct TilesetEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, tileset: &mut Tileset) {
        self.editor.image_editor.drop_selection(tileset);
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        self.dialogs.show(wc, &mut self.editor, tileset);

        let title = format!("{} - Tileset", tileset.asset.name);
        let window = super::DataAssetEditor::create_window(&mut self.asset, wc, &title);
        let (min_size, default_size) = DataAssetEditor::calc_image_editor_window_size(tileset);
        window.min_size(min_size).default_size(default_size).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, tileset);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    add_tiles_dialog: AddTilesDialog,
    rm_tiles_dialog: RemoveTilesDialog,
    import_dialog: ImportDialog,
    export_dialog: ExportDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_tiles_dialog: AddTilesDialog::new(),
            rm_tiles_dialog: RemoveTilesDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
       }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, tileset: &mut Tileset) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, tileset) {
            Editor::reload_images(wc, tileset);
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(tileset.num_tiles-1);
            editor.image_editor.set_selected_image(editor.image_picker.selected_image, tileset);
            editor.image_editor.set_undo_target(tileset);
        }
        if self.add_tiles_dialog.open && self.add_tiles_dialog.show(wc, tileset) {
            Editor::reload_images(wc, tileset);
            editor.image_editor.set_undo_target(tileset);
        }
        if self.rm_tiles_dialog.open && self.rm_tiles_dialog.show(wc, tileset) {
            Editor::reload_images(wc, tileset);
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(tileset.num_tiles-1);
            editor.image_editor.set_selected_image(editor.image_picker.selected_image, tileset);
            editor.image_editor.set_undo_target(tileset);
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, tileset);
        }
        if self.import_dialog.open && self.import_dialog.show(wc, tileset) {
            editor.image_picker.selected_image = editor.image_picker.selected_image.min(tileset.num_tiles-1);
            if ! editor.image_editor.set_selected_image(editor.image_picker.selected_image, tileset) {
                editor.image_editor.set_undo_target(tileset);
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
    fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            color_picker: ColorPickerWidget::new(0b000011, 0b110000),
            image_picker: ImagePickerWidget::new(),
            image_editor: ImageEditorWidget::new(),
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, tileset: &mut Tileset) {
        let import_tile_dlg_id = format!("editor_{}_import_tile", tileset.asset.id);
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&import_tile_dlg_id) {
            let image = match ImagePixels::load_png(&filename) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", filename.display(), e));
                    return;
                }
            };
            self.image_editor.paste_pixels(tileset, image);
        }

        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tileset", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Import...").clicked() {
                            dialogs.import_dialog.set_open(wc);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            dialogs.export_dialog.set_open(wc, tileset);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, tileset, self.color_picker.right_color);
                        }
                    });
                });
                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.undo).max_width(14.0).max_height(14.0));
                        if ui.button("Undo").clicked() {
                            self.image_editor.undo(tileset);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if self.image_editor.selection.is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.cut).max_width(14.0).max_height(14.0));
                        if ui.button("Cut").clicked() {
                            self.image_editor.cut(wc, tileset, self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        if self.image_editor.selection.is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.copy).max_width(14.0).max_height(14.0));
                        if ui.button("Copy").clicked() {
                            self.image_editor.copy(wc, tileset);
                        }
                    });
                    ui.horizontal(|ui| {
                        if wc.image_clipboard.is_none() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.paste).max_width(14.0).max_height(14.0));
                        if ui.button("Paste").clicked() {
                            self.image_editor.paste(wc, tileset);
                        }
                    });
                    ui.horizontal(|ui| {
                        if self.image_editor.selection.is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Delete selection").clicked() {
                            self.image_editor.delete_selection(tileset, self.color_picker.right_color);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Paste from file...").clicked() {
                            wc.sys_dialogs.open_file(
                                Some(wc.egui.window),
                                import_tile_dlg_id,
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
                        if ui.button("Insert tiles...").clicked() {
                            dialogs.add_tiles_dialog.set_open(wc, AddTilesAction::Insert, self.image_picker.selected_image,
                                                              self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.add).max_width(14.0).max_height(14.0));
                        if ui.button("Append tiles...").clicked() {
                            dialogs.add_tiles_dialog.set_open(wc, AddTilesAction::Append, self.image_picker.selected_image,
                                                              self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Remove tiles...").clicked() {
                            dialogs.rm_tiles_dialog.set_open(wc, tileset, self.image_picker.selected_image);
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, tileset: &mut Tileset) {
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
                    self.image_editor.vflip(tileset, self.color_picker.right_color);
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Horizontal Flip").clicked() {
                    self.image_editor.hflip(tileset, self.color_picker.right_color);
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

    fn reload_images(wc: &mut WindowContext, asset: &impl ImageCollection) {
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Opaque, true);
        asset.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, true);
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, tileset: &mut Tileset) {
        self.show_menu_bar(ui, wc, dialogs, tileset);
        self.show_toolbar(ui, wc, tileset);

        // footer:
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.label(format!("{} bytes [{} tiles]", tileset.data_size(), tileset.num_tiles));
        });

        // item picker:
        egui::SidePanel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.image_picker.zoom = 4.0;
            self.image_picker.display = self.image_editor.display;
            let texture = tileset.texture(wc.tex_man, wc.egui.ctx, self.image_picker.display.texture_slot());
            self.image_picker.show(ui, wc.settings, tileset, texture);
            self.image_editor.set_selected_image(self.image_picker.selected_image, tileset);
        });

        // color picker:
        egui::SidePanel::right(format!("editor_panel_{}_right", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.color_picker.show(ui, wc);
        });

        // image:
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let colors = (self.color_picker.left_color, self.color_picker.right_color);
            self.image_editor.show(ui, wc, tileset, colors);
            self.color_picker.maybe_set_colors(
                self.image_editor.pick_left_color.take(),
                self.image_editor.pick_right_color.take()
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            self.image_editor.handle_keyboard(ui, wc, tileset, self.color_picker.right_color);
        }
    }
}
