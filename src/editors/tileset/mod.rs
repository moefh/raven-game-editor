mod properties;
mod add_tiles;
mod remove_tiles;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::{Tileset, DataAssetId, GenericAsset, ImageCollectionAsset};

use properties::PropertiesDialog;
use remove_tiles::RemoveTilesDialog;
use add_tiles::{AddTilesDialog, AddTilesAction};
use super::widgets::{ColorPickerState, ImagePickerState, ImageEditorState, ImageDrawingTool, ImageDisplay};

pub struct TilesetEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: PropertiesDialog,
    add_tiles_dialog: AddTilesDialog,
    rm_tiles_dialog: RemoveTilesDialog,
    color_picker: ColorPickerState,
    image_picker: ImagePickerState,
    image_editor: ImageEditorState,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: PropertiesDialog::new(),
            add_tiles_dialog: AddTilesDialog::new(),
            rm_tiles_dialog: RemoveTilesDialog::new(),
            color_picker: ColorPickerState::new(0b000011, 0b110000),
            image_picker: ImagePickerState::new(),
            image_editor: ImageEditorState::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, tileset: &mut Tileset) {
        self.image_editor.drop_selection(tileset);
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, tileset: &mut Tileset) {
        let asset_id = tileset.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tileset", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            self.properties_dialog.set_open(tileset, self.color_picker.right_color);
                        }
                    });
                });
                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.new).max_width(14.0).max_height(14.0));
                        if ui.button("Insert tiles...").clicked() {
                            self.add_tiles_dialog.set_open(AddTilesAction::Insert, self.image_picker.selected_image,
                                                           self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.new).max_width(14.0).max_height(14.0));
                        if ui.button("Append tiles...").clicked() {
                            self.add_tiles_dialog.set_open(AddTilesAction::Append, self.image_picker.selected_image,
                                                           self.color_picker.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Remove tiles...").clicked() {
                            self.rm_tiles_dialog.set_open(tileset, self.image_picker.selected_image);
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, tileset: &mut Tileset) {
        let asset_id = tileset.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_toolbar", asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Tool:");
                ui.add_space(1.0);
                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.image_editor.tool == ImageDrawingTool::Pencil)
                          .frame_when_inactive(self.image_editor.tool == ImageDrawingTool::Pencil)).on_hover_text("Pencil").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Pencil);
                }

                if ui.add(egui::Button::image(IMAGES.fill)
                          .selected(self.image_editor.tool == ImageDrawingTool::Fill)
                          .frame_when_inactive(self.image_editor.tool == ImageDrawingTool::Fill)).on_hover_text("Fill").clicked() {
                    self.image_editor.set_tool(ImageDrawingTool::Fill);
                }

                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.image_editor.tool == ImageDrawingTool::Select)
                          .frame_when_inactive(self.image_editor.tool == ImageDrawingTool::Select)).on_hover_text("Select").clicked() {
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

    pub fn reload_images(wc: &mut WindowContext, asset: &impl ImageCollectionAsset) {
        ImageCollection::plus_loaded_texture(asset, wc.tex_man, wc.egui.ctx, TextureSlot::Opaque, true);
        ImageCollection::plus_loaded_texture(asset, wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, true);
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, tileset) {
            self.image_picker.selected_image = self.image_picker.selected_image.min(tileset.num_tiles-1);
            self.image_editor.selected_image = self.image_picker.selected_image;
            Self::reload_images(wc, tileset);
        }
        if self.add_tiles_dialog.open && self.add_tiles_dialog.show(wc, tileset) {
            Self::reload_images(wc, tileset);
        }
        if self.rm_tiles_dialog.open && self.rm_tiles_dialog.show(wc, tileset) {
            self.image_picker.selected_image = self.image_picker.selected_image.min(tileset.num_tiles-1);
            self.image_editor.selected_image = self.image_picker.selected_image;
            Self::reload_images(wc, tileset);
        }

        let asset_id = tileset.asset.id;
        let title = format!("{} - Tileset", tileset.asset.name);
        let window = super::create_editor_window(self.asset.id, &title, wc);
        let (min_size, default_size) = super::calc_image_editor_window_size(tileset);
        let mut asset_open = self.asset.open;
        window.min_size(min_size).default_size(default_size).open(&mut asset_open).show(wc.egui.ctx, |ui| {
            self.show_menu_bar(ui, wc, tileset);
            self.show_toolbar(ui, wc, tileset);

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", tileset.data_size()));
            });

            // item picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                self.image_picker.zoom = 4.0;
                self.image_picker.display = self.image_editor.display;
                let (image, texture) = ImageCollection::plus_texture(tileset, wc.tex_man, wc.egui.ctx, self.image_picker.display.texture_slot());
                super::widgets::image_picker(ui, wc.settings, &image, texture, &mut self.image_picker);
                self.image_editor.selected_image = self.image_picker.selected_image;
            });

            // color picker:
            egui::SidePanel::right(format!("editor_panel_{}_right", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                super::widgets::color_picker(ui, wc, &mut self.color_picker);
            });

            // image:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let colors = (self.color_picker.left_color, self.color_picker.right_color);
                super::widgets::image_editor(ui, wc, tileset, &mut self.image_editor, colors);
            });
        });
        self.asset.open = asset_open;
    }
}
