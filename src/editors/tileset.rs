use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{Tileset, DataAssetId, GenericAsset};

struct PropertiesDialog {
    image_changed: bool,
    open: bool,
    name: String,
    num_tiles: f32,
    sel_color: u8,
}

impl PropertiesDialog {
    fn new() -> Self {
        PropertiesDialog {
            image_changed: false,
            open: false,
            name: String::new(),
            num_tiles: 0.0,
            sel_color: 0,
        }
    }

    fn set_open(&mut self, tileset: &Tileset, sel_color: u8) {
        self.name.clear();
        self.name.push_str(&tileset.asset.name);
        self.num_tiles = tileset.num_tiles as f32;
        self.sel_color = sel_color;
        self.open = true;
    }

    fn confirm(&mut self, tileset: &mut Tileset) {
        tileset.asset.name.clear();
        tileset.asset.name.push_str(&self.name);
        if self.num_tiles as u32 != tileset.num_tiles {
            let image = ImageCollection::from_asset(tileset);
            image.resize(tileset.width, tileset.height, self.num_tiles as u32, &mut tileset.data, self.sel_color);
            tileset.num_tiles = self.num_tiles as u32;
            self.image_changed = true;
        }
    }

    fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) -> bool {
        if egui::Modal::new(egui::Id::new("dlg_about")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Tileset Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", tileset.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Num tiles:");
                        ui.add(egui::Slider::new(&mut self.num_tiles, 1.0..=255.0).step_by(1.0));
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(tileset);
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

pub struct TilesetEditor {
    pub asset: super::DataAssetEditor,
    force_reload_image: bool,
    properties_dialog: PropertiesDialog,
    selected_tile: u32,
    color_picker: super::widgets::ColorPickerState,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            asset: super::DataAssetEditor::new(id, open),
            force_reload_image: false,
            properties_dialog: PropertiesDialog::new(),
            selected_tile: 0,
            color_picker: super::widgets::ColorPickerState::new(0b000011, 0b110000),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, tileset) {
            self.selected_tile = self.selected_tile.min(tileset.num_tiles-1);
            self.force_reload_image = true;
        }

        let asset_id = tileset.asset.id;
        let title = format!("{} - Tileset", tileset.asset.name);
        let window = super::create_editor_window(self.asset.id, &title, wc);
        let (min_size, default_size) = super::calc_image_editor_window_size(tileset);
        window.min_size(min_size).default_size(default_size).open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
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
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", tileset.data_size()));
            });

            let (image, texture) = ImageCollection::load_asset(tileset, wc.tex_man, wc.egui.ctx, self.force_reload_image);
            self.force_reload_image = false;

            // item picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                let picker_zoom = 4.0;
                let scroll = super::widgets::image_item_picker(ui, asset_id, texture, &image, self.selected_tile, picker_zoom);
                if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                    let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                    if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                        let frame_size = picker_zoom * image.get_item_size();
                        self.selected_tile = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                    }
                };
            });

            // color picker:
            egui::SidePanel::right(format!("editor_panel_{}_right", asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                super::widgets::color_picker(ui, asset_id, &mut self.color_picker);
            });

            // image:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt(format!("tileset_{}_tiles", asset_id)), |ui| {
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
