use crate::IMAGES;
use crate::app::{WindowContext, ImageCollection};
use crate::data_asset::{Tileset, DataAssetId, GenericAsset};

pub struct TilesetEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    selected_tile: u32,
    color_picker: super::widgets::ColorPickerState,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            selected_tile: 0,
            color_picker: super::widgets::ColorPickerState::new(0b000011, 0b110000),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        let title = format!("{} - Tileset", tileset.asset.name);
        let window = super::create_editor_window(self.asset.id, &title, wc);
        let (min_size, default_size) = super::calc_image_editor_window_size(tileset);
        window.min_size(min_size).default_size(default_size).open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", tileset.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Tileset", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                //...
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", tileset.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", tileset.data_size()));
            });

            let (image, texture) = ImageCollection::load_asset(tileset, wc.tex_man, wc.egui.ctx, self.force_reload_image);
            self.force_reload_image = false;

            // item picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", tileset.asset.id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let picker_zoom = 4.0;
                let scroll = super::widgets::image_item_picker(ui, tileset.asset.id, texture, &image, self.selected_tile, picker_zoom);
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        let frame_size = picker_zoom * image.get_item_size();
                        self.selected_tile = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                    }
                };
            });

            // color picker:
            egui::SidePanel::right(format!("editor_panel_{}_right", tileset.asset.id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                super::widgets::color_picker(ui, tileset.asset.id, &mut self.color_picker);
            });

            // image:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt(format!("tileset_{}_tiles", tileset.asset.id)), |ui| {
                        let (resp, canvas_to_image) = super::widgets::image_editor(ui, texture, &image, self.selected_tile);
                        if let Some(pointer_pos) = resp.interact_pointer_pos() &&
                            canvas_to_image.from().contains(pointer_pos) {
                                let image_pos = canvas_to_image * pointer_pos;
                                let x = image_pos.x as i32;
                                let y = image_pos.y as i32;
                                if let Some(color) = if resp.dragged_by(egui::PointerButton::Primary) {
                                    Some(self.color_picker.left_color)
                                } else if resp.dragged_by(egui::PointerButton::Secondary) {
                                    Some(self.color_picker.right_color)
                                } else {
                                    None
                                } {
                                    self.force_reload_image = image.set_pixel(&mut tileset.data, x, y, self.selected_tile, color);
                                }
                            }
                    });
            });
        });
    }
}
