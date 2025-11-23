mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::misc::ImageCollection;
use crate::data_asset::{Tileset, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::widgets::{ColorPickerState, ImageEditorState, ImageDrawingTool, ImageDisplay};

pub struct TilesetEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    properties_dialog: PropertiesDialog,
    color_picker: ColorPickerState,
    image_editor: ImageEditorState,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            properties_dialog: PropertiesDialog::new(),
            color_picker: ColorPickerState::new(0b000011, 0b110000),
            image_editor: ImageEditorState::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, tileset: &mut Tileset) {
        self.image_editor.drop_selection(tileset);
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, tile: &mut Tileset) {
        let asset_id = tile.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tile", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            self.properties_dialog.set_open(tile, self.color_picker.right_color);
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
                    self.force_reload_image = true;
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Horizontal Flip").clicked() {
                    self.image_editor.hflip(tileset, self.color_picker.right_color);
                    self.force_reload_image = true;
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

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, tileset) {
            self.image_editor.selected_image = self.image_editor.selected_image.min(tileset.num_tiles-1);
            self.force_reload_image = true;
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
                let picker_zoom = 4.0;
                let slot = self.image_editor.display.texture_slot();
                let (image, texture) = ImageCollection::load_asset_texture(tileset, wc.tex_man, wc.egui.ctx, slot, self.force_reload_image);
                self.force_reload_image = false;
                let scroll = super::widgets::image_item_picker(ui, texture, &image, self.image_editor.selected_image, picker_zoom);
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        let frame_size = picker_zoom * image.get_item_size();
                        self.image_editor.selected_image = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                    }
                };
            });

            // color picker:
            egui::SidePanel::right(format!("editor_panel_{}_right", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                super::widgets::color_picker(ui, &mut self.color_picker);
            });

            // image:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let colors = (self.color_picker.left_color, self.color_picker.right_color);
                super::widgets::image_editor(ui, wc.tex_man, tileset, &mut self.image_editor, colors);
            });
        });
        self.asset.open = asset_open;
    }
}
