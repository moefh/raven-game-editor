use crate::IMAGES;
use crate::app::{WindowContext, ImageCollection};
use crate::data_asset::{Sprite, DataAssetId, GenericAsset};

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    selected_frame: u32,
    color_picker: super::widgets::ColorPickerState,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            selected_frame: 0,
            color_picker: super::widgets::ColorPickerState::new(0b000011, 0b001100),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        let title = format!("{} - Sprite", sprite.asset.name);
        let window = super::create_editor_window(sprite.asset.id, &title, wc);
        let (min_size, default_size) = super::calc_image_editor_window_size(sprite);
        window.min_size(min_size).default_size(default_size).open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", sprite.asset.id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Sprite", |ui| {
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
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", sprite.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", sprite.data_size()));
            });

            let (image, texture) = ImageCollection::load_asset(sprite, wc.tex_man, wc.egui.ctx, self.force_reload_image);
            self.force_reload_image = false;

            // item picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", sprite.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let picker_zoom = 2.0;
                let scroll = super::widgets::image_item_picker(ui, sprite.asset.id, texture, &image, self.selected_frame, picker_zoom);
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        let frame_size = picker_zoom * image.get_item_size();
                        self.selected_frame = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                    }
                };
            });

            // color picker:
            egui::SidePanel::right(format!("editor_panel_{}_right", sprite.asset.id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                super::widgets::color_picker(ui, sprite.asset.id, &mut self.color_picker);
            });

            // image:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt(format!("sprite_{}_frames", sprite.asset.id)), |ui| {
                        let (resp, canvas_to_image) = super::widgets::image_editor(ui, texture, &image, self.selected_frame);
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
                                    self.force_reload_image = image.set_pixel(&mut sprite.data, x, y, self.selected_frame, color);
                                }
                            }
                    });
            });
        });
    }
}
