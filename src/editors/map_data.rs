use crate::IMAGES;
use crate::misc::{WindowContext, ImageCollection};
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId, GenericAsset};

struct PropertiesDialog {
    open: bool,
    name: String,
    tileset_id: DataAssetId,
    width: f32,
    height: f32,
    bg_width: f32,
    bg_height: f32,
    new_tile: u8,
}

/*
fn resize_map(map_data: &mut MapData, new_w: u32, new_h: u32, new_bg_w: u32, new_bg_h: u32, new_tile: u8) {
    // TODO

}
*/

impl PropertiesDialog {
    fn new(tileset_id: DataAssetId) -> Self {
        PropertiesDialog {
            open: false,
            name: String::new(),
            tileset_id,
            width: 0.0,
            height: 0.0,
            bg_width: 0.0,
            bg_height: 0.0,
            new_tile: 0,
        }
    }

    fn set_open(&mut self, map_data: &MapData, new_tile: u8) {
        self.name.clear();
        self.name.push_str(&map_data.asset.name);
        self.tileset_id = map_data.tileset_id;
        self.width = map_data.width as f32;
        self.height = map_data.height as f32;
        self.bg_width = map_data.bg_width as f32;
        self.bg_height = map_data.bg_height as f32;
        self.new_tile = new_tile;
        self.open = true;
    }

    fn confirm(&mut self, map_data: &mut MapData) {
        map_data.asset.name.clear();
        map_data.asset.name.push_str(&self.name);
        map_data.tileset_id = self.tileset_id;

        let width = self.width as u32;
        let height = self.height as u32;
        let bg_width = self.width as u32;
        let bg_height = self.height as u32;
        if width != map_data.width || height != map_data.height || bg_width != map_data.bg_width || bg_height != map_data.bg_height {
            //resize_map(map_data, width, height, bg_width, bg_height, self.new_tile);  <-- TODO
            map_data.width = width;
            map_data.height = height;
            map_data.bg_width = bg_width;
            map_data.bg_height = bg_height;
        }
    }

    fn show(&mut self, wc: &WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if ! self.open { return; }

        if egui::Modal::new(egui::Id::new("dlg_map_data_properties")).show(wc.egui.ctx, |ui| {
            ui.set_width(250.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Map Properties");
                ui.add_space(16.0);

                egui::Grid::new(format!("editor_panel_{}_prop_grid", map_data.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Tileset:");
                        let cur_tileset_name = if let Some(cur_tileset) = tilesets.get(&self.tileset_id) {
                            &cur_tileset.asset.name
                        } else {
                            "??"
                        };
                        egui::ComboBox::from_id_salt(format!("map_editor_tileset_combo_{}", map_data.asset.id))
                            .selected_text(cur_tileset_name)
                            .show_ui(ui, |ui| {
                                for tileset_id in tileset_ids.iter() {
                                    if let Some(tileset) = tilesets.get(tileset_id) {
                                        ui.selectable_value(&mut self.tileset_id, tileset.asset.id, &tileset.asset.name);
                                    }
                                }
                            });
                        ui.end_row();

                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1.0..=512.0).step_by(1.0));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1.0..=512.0).step_by(1.0));
                        ui.end_row();

                        ui.label("BG Width:");
                        ui.add(egui::Slider::new(&mut self.width, 1.0..=512.0).step_by(1.0));
                        ui.end_row();

                        ui.label("BG Height:");
                        ui.add(egui::Slider::new(&mut self.height, 1.0..=512.0).step_by(1.0));
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() {
                        self.confirm(map_data);
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
        }
    }
}

pub struct MapDataEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: Option<PropertiesDialog>,
    left_tile: u32,
    right_tile: u32,
    state: super::widgets::MapEditorState,
}

fn calc_map_editor_window_size() -> (egui::Vec2, egui::Vec2) {
    let min_size = egui::Vec2::new(130.0 + 160.0, 80.0 + 100.0);
    let default_size = egui::Vec2::new(130.0 + 500.0, 80.0 + 300.0);
    (min_size, default_size)
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: None,
            left_tile: 0,
            right_tile: 0,
            state: super::widgets::MapEditorState::new(),
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, tileset_ids, tilesets);
        }

        let asset_id = map_data.asset.id;
        let title = format!("{} - Map", map_data.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let (min_size, default_size) = calc_map_editor_window_size();
        window.min_size(min_size).default_size(default_size).open(&mut self.asset.open).show(wc.egui.ctx, |ui| {
            // header:
            egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Map", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                            if ui.button("Properties...").clicked() {
                                let dlg = self.properties_dialog.get_or_insert_with(|| {
                                    PropertiesDialog::new(map_data.tileset_id)
                                });
                                dlg.set_open(map_data, self.right_tile as u8);
                            }
                        });
                    });
                });
            });

            // footer:
            egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.label(format!("{} bytes", map_data.data_size()));
            });

            if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
                let (image, texture) = ImageCollection::load_asset(tileset, wc.tex_man, wc.egui.ctx, false);

                // tile picker:
                egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                    ui.add_space(5.0);
                    let picker_zoom = 4.0;
                    let scroll = super::widgets::image_item_picker(ui, texture, &image, self.left_tile, picker_zoom);
                    if let Some(pointer_pos) = scroll.inner.interact_pointer_pos() {
                        let pos = pointer_pos - scroll.inner_rect.min + scroll.state.offset;
                        if pos.x >= 0.0 && pos.x <= scroll.inner_rect.width() {
                            let frame_size = picker_zoom * image.get_item_size();
                            let sel_tile = u32::min((pos.y / frame_size.y).floor() as u32, image.num_items-1);
                            if scroll.inner.dragged_by(egui::PointerButton::Primary) {
                                self.left_tile = sel_tile;
                            } else if scroll.inner.dragged_by(egui::PointerButton::Secondary) {
                                self.right_tile = sel_tile;
                            }
                        }
                    };
                });

                // body:
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    super::widgets::map_editor(ui, map_data, texture, &image, &mut self.state);
                });
            }
        });
    }
}
