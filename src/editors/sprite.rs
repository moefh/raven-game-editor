use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{Sprite, DataAssetId, GenericAsset};

struct PropertiesDialog {
    image_changed: bool,
    open: bool,
    name: String,
    width: f32,
    height: f32,
    num_frames: f32,
    sel_color: u8,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            width: 0.0,
            height: 0.0,
            num_frames: 0.0,
            sel_color: 0,
        }
    }

    fn set_open(&mut self, sprite: &Sprite, sel_color: u8) {
        self.name.clear();
        self.name.push_str(&sprite.asset.name);
        self.width = sprite.width as f32;
        self.height = sprite.height as f32;
        self.num_frames = sprite.num_frames as f32;
        self.sel_color = sel_color;
        self.open = true;
    }

    fn confirm(&mut self, sprite: &mut Sprite) {
        sprite.asset.name.clear();
        sprite.asset.name.push_str(&self.name);

        let width = self.width as u32;
        let height = self.height as u32;
        let num_frames = self.num_frames as u32;
        if num_frames != sprite.num_frames || width != sprite.width || height != sprite.height {
            let image = ImageCollection::from_asset(sprite);
            image.resize(width, height, num_frames, &mut sprite.data, self.sel_color);
            sprite.width = width;
            sprite.height = height;
            sprite.num_frames = num_frames;
            self.image_changed = true;
        }
    }

    fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if ! self.open { return false; }

        if egui::Modal::new(egui::Id::new("dlg_about")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Sprite Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1.0..=512.0).step_by(1.0));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1.0..=512.0).step_by(1.0));
                        ui.end_row();

                        ui.label("Num frames:");
                        ui.add(egui::Slider::new(&mut self.num_frames, 1.0..=255.0).step_by(1.0));
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(sprite);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}

pub struct SpriteEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    properties_dialog: PropertiesDialog,
    selected_frame: u32,
    color_picker: super::widgets::ColorPickerState,
}

impl SpriteEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        SpriteEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            properties_dialog: PropertiesDialog::new(),
            selected_frame: 0,
            color_picker: super::widgets::ColorPickerState::new(0b000011, 0b001100),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, sprite) {
            self.selected_frame = self.selected_frame.min(sprite.num_frames-1);
            self.force_reload_image = true;
        }

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
                                self.properties_dialog.set_open(sprite, self.color_picker.right_color);
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
            egui::SidePanel::left(format!("editor_panel_{}_left", sprite.asset.id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let picker_zoom = if sprite.width > 100 { 1.0 } else { 2.0 };
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
            egui::SidePanel::right(format!("editor_panel_{}_right", sprite.asset.id)).resizable(false).show_inside(ui, |ui| {
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
