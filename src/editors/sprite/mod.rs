mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::misc::{ImageCollection, TextureSlot};
use crate::data_asset::{Sprite, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::{DrawingTool, ImageDisplay};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    properties_dialog: PropertiesDialog,
    selected_frame: u32,
    color_picker: super::widgets::ColorPickerState,
    tool: DrawingTool,
    display: u32,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            properties_dialog: PropertiesDialog::new(),
            selected_frame: 0,
            color_picker: super::widgets::ColorPickerState::new(0b000011, 0b001100),
            tool: DrawingTool::Pencil,
            display: ImageDisplay::TRANSPARENT | ImageDisplay::GRID,
        }
    }

    fn get_selected_color_for_click(&self, resp: &egui::Response) -> Option<u8> {
        if resp.dragged_by(egui::PointerButton::Primary) {
            Some(self.color_picker.left_color)
        } else if resp.dragged_by(egui::PointerButton::Secondary) {
            Some(self.color_picker.right_color)
        } else {
            None
        }
    }

    fn handle_click(&mut self, image: &ImageCollection, x: i32, y: i32, sprite: &mut Sprite, resp: &egui::Response) {
        match self.tool {
            DrawingTool::Pencil => if let Some(color) = self.get_selected_color_for_click(resp) {
                self.force_reload_image = image.set_pixel(&mut sprite.data, x, y, self.selected_frame, color);
            }

            DrawingTool::Fill => if let Some(color) = self.get_selected_color_for_click(resp) {
                self.force_reload_image = image.flood_fill(&mut sprite.data, x, y, self.selected_frame, color);
            }

            DrawingTool::Select => {
                // TODO
            }
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, sprite: &mut Sprite) {
        let asset_id = sprite.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Sprite", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            self.properties_dialog.set_open(sprite, self.color_picker.right_color);
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, sprite: &mut Sprite) {
        let asset_id = sprite.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_toolbar", asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Tool:");
                ui.add_space(1.0);
                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.tool == DrawingTool::Pencil)
                          .frame_when_inactive(self.tool == DrawingTool::Pencil)).on_hover_text("Pencil").clicked() {
                    self.tool = DrawingTool::Pencil;
                }
                if ui.add(egui::Button::image(IMAGES.fill)
                          .selected(self.tool == DrawingTool::Fill)
                          .frame_when_inactive(self.tool == DrawingTool::Fill)).on_hover_text("Fill").clicked() {
                    self.tool = DrawingTool::Fill;
                }
                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.tool == DrawingTool::Select)
                          .frame_when_inactive(self.tool == DrawingTool::Select)).on_hover_text("Select").clicked() {
                    self.tool = DrawingTool::Select;
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(egui::Button::image(IMAGES.v_flip)).on_hover_text("Vertical Flip").clicked() {
                    let image = ImageCollection::from_asset(sprite);
                    image.v_flip(&mut sprite.data, self.selected_frame);
                    self.force_reload_image = true;
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Horizontal Flip").clicked() {
                    let image = ImageCollection::from_asset(sprite);
                    image.h_flip(&mut sprite.data, self.selected_frame);
                    self.force_reload_image = true;
                }
                ui.spacing_mut().item_spacing = spacing;

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.grid)
                                  .selected((self.display & ImageDisplay::GRID) != 0)
                                  .frame_when_inactive((self.display & ImageDisplay::GRID) != 0))
                            .on_hover_text("Grid").clicked() {
                                self.display ^= ImageDisplay::GRID;
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                                  .selected((self.display & ImageDisplay::TRANSPARENT) != 0)
                                  .frame_when_inactive((self.display & ImageDisplay::TRANSPARENT) != 0))
                            .on_hover_text("Transparency").clicked() {
                                self.display ^= ImageDisplay::TRANSPARENT;
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

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, sprite) {
            self.selected_frame = self.selected_frame.min(sprite.num_frames-1);
            self.force_reload_image = true;
        }

        let asset_id = sprite.asset.id;
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let (min_size, default_size) = super::calc_image_editor_window_size(sprite);
        let mut asset_open = self.asset.open;
        window.min_size(min_size).default_size(default_size).open(&mut asset_open).show(wc.egui.ctx, |ui| {
            self.show_menu_bar(ui, wc, sprite);
            self.show_toolbar(ui, wc, sprite);

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", sprite.data_size()));
            });

            let slot = if (self.display & ImageDisplay::TRANSPARENT) == 0 { TextureSlot::Opaque } else { TextureSlot::Transparent };
            let (image, texture) = ImageCollection::load_asset_texture(sprite, wc.tex_man, wc.egui.ctx, slot, self.force_reload_image);
            self.force_reload_image = false;

            // item picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let picker_zoom = if sprite.width > 100 { 1.0 } else { 2.0 };
                let scroll = super::widgets::image_item_picker(ui, texture, &image, self.selected_frame, picker_zoom);
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        let frame_size = picker_zoom * image.get_item_size();
                        self.selected_frame = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
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
                let (resp, canvas_to_image) = super::widgets::image_editor(ui, texture, &image, self.selected_frame, self.display);
                if let Some(pointer_pos) = resp.interact_pointer_pos() &&
                    canvas_to_image.from().contains(pointer_pos) {
                        let image_pos = canvas_to_image * pointer_pos;
                        let x = image_pos.x as i32;
                        let y = image_pos.y as i32;
                        self.handle_click(&image, x, y, sprite, &resp);
                    }
            });
        });
        self.asset.open = asset_open;
    }
}
