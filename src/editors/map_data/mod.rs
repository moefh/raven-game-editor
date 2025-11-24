mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
//use crate::misc::{ImageCollection, TextureSlot};
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId, GenericAsset};

use properties::PropertiesDialog;
use super::widgets::{MapEditorState, MapDisplay, ImagePickerState};

const ZOOM_OPTIONS: &[f32] = &[ 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0 ];

fn calc_map_editor_window_size() -> (egui::Vec2, egui::Vec2) {
    let min_size = egui::Vec2::new(500.0, 200.0);
    let default_size = egui::Vec2::new(630.0, 380.0);
    (min_size, default_size)
}

pub struct MapDataEditor {
    pub asset: super::DataAssetEditor,
    properties_dialog: Option<PropertiesDialog>,
    map_editor: MapEditorState,
    image_picker: ImagePickerState,
    use_custom_grid_color: bool,
    custom_grid_color: egui::Color32,
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            asset: super::DataAssetEditor::new(id, open),
            properties_dialog: None,
            map_editor: MapEditorState::new(),
            image_picker: ImagePickerState::new().use_as_palette(true),
            use_custom_grid_color: false,
            custom_grid_color: egui::Color32::RED,
        }
    }

    pub fn prepare_for_saving(&mut self, _asset: &mut impl crate::data_asset::GenericAsset) {
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, map_data: &MapData) {
        let asset_id = map_data.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Map", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            let dlg = self.properties_dialog.get_or_insert_with(|| {
                                PropertiesDialog::new(map_data.tileset_id)
                            });
                            dlg.set_open(map_data, self.image_picker.selected_image_right as u8);
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, map_data: &mut MapData) {
        let asset_id = map_data.asset.id;
        egui::TopBottomPanel::top(format!("editor_panel_{}_toolbar", asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Display:");
                ui.add_space(1.0);

                if ui.add(egui::Button::image(IMAGES.layer_fg)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::FOREGROUND))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::FOREGROUND)))
                    .on_hover_text("Foreground").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::FOREGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_bg)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::BACKGROUND))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::BACKGROUND)))
                    .on_hover_text("Background").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::BACKGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_clip)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::CLIP))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::CLIP)))
                    .on_hover_text("Collision").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::CLIP);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_fx)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::EFFECTS))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::EFFECTS)))
                    .on_hover_text("Effects").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::EFFECTS);
                    }

                if ui.add(egui::Button::image(IMAGES.grid)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::GRID))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::GRID)))
                    .on_hover_text("Grid").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::GRID);
                    }

                if ui.add(egui::Button::image(IMAGES.screen)
                          .selected(self.map_editor.display_layers.has_bits(MapDisplay::SCREEN_SIZE))
                          .frame_when_inactive(self.map_editor.display_layers.has_bits(MapDisplay::SCREEN_SIZE)))
                    .on_hover_text("Screen").clicked() {
                        self.map_editor.display_layers.toggle(MapDisplay::SCREEN_SIZE);
                    }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.label("Zoom:");
                ui.add_space(1.0);
                let cur_zoom_name = if let Some(zoom) = ZOOM_OPTIONS.iter().find(|z| **z == self.map_editor.zoom) {
                    &format!("{:3.1}x", zoom)
                } else {
                    "custom"
                };
                egui::ComboBox::from_id_salt(format!("map_editor_{}_zoom_combo", asset_id))
                    .selected_text(cur_zoom_name)
                    .width(75.0)
                    .show_ui(ui, |ui| {
                        for zoom in ZOOM_OPTIONS {
                            ui.selectable_value(&mut self.map_editor.zoom, *zoom, format!("{:3.1}x", zoom));
                        }
                    });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);


                if ui.add(egui::Button::new("Grid Color").selected(self.use_custom_grid_color)/*.frame_when_inactive(true)*/).clicked() {
                    self.use_custom_grid_color = ! self.use_custom_grid_color;
                }
                ui.add_space(2.0);
                if self.use_custom_grid_color {
                    let mut rgba = self.custom_grid_color.into();
                    egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                    self.custom_grid_color = rgba.into();
                    self.map_editor.custom_grid_color = Some(self.custom_grid_color);
                } else {
                    ui.add_space(2.0);
                    ui.label("default");
                    self.map_editor.custom_grid_color = None;
                }

                ui.spacing_mut().item_spacing = spacing;
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_footer(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, map_data: &mut MapData) {
        let asset_id = map_data.asset.id;
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(format!("{} bytes", map_data.data_size()));
                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        ui.add_space(1.0);
                        ui.label(format!("({}, {})", self.map_editor.hover_pos.x.floor(), self.map_editor.hover_pos.y.floor()));
                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, tileset_ids, tilesets);
        }

        let asset_id = map_data.asset.id;
        let title = format!("{} - Map", map_data.asset.name);
        let window = super::create_editor_window(asset_id, &title, wc);
        let (min_size, default_size) = calc_map_editor_window_size();
        let mut asset_open = self.asset.open;
        window.min_size(min_size).default_size(default_size).open(&mut asset_open).show(wc.egui.ctx, |ui| {
            self.show_menubar(ui, map_data);
            self.show_toolbar(ui, wc, map_data);
            self.show_footer(ui, wc, map_data);

            if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
                // tile picker:
                egui::SidePanel::left(format!("editor_panel_{}_left", asset_id)).resizable(false).show_inside(ui, |ui| {
                    ui.add_space(5.0);
                    self.image_picker.zoom = 4.0;
                    super::widgets::image_picker(ui, wc, tileset, &mut self.image_picker);
                    self.map_editor.left_draw_tile = self.image_picker.selected_image;
                    self.map_editor.right_draw_tile = self.image_picker.selected_image_right;
                });

                // body:
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    super::widgets::map_editor(ui, wc, map_data, tileset, &mut self.map_editor);
                });
            }
        });
        self.asset.open = asset_open;
    }
}
