mod properties;

use crate::IMAGES;
use crate::app::WindowContext;
use crate::image::{ImageCollection, TextureSlot};
use crate::data_asset::{MapData, Tileset, AssetIdList, AssetList, DataAssetId, GenericAsset};
use crate::misc::STATIC_IMAGES;

use properties::PropertiesDialog;
use super::DataAssetEditor;
use super::widgets::{MapEditorWidget, MapDisplay, MapLayer, MapTool, ImagePickerWidget};

const ZOOM_OPTIONS: &[f32] = &[ 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0 ];

fn calc_map_editor_window_size() -> (egui::Vec2, egui::Vec2) {
    let min_size = egui::Vec2::new(500.0, 200.0);
    let default_size = egui::Vec2::new(630.0, 380.0);
    (min_size, default_size)
}

pub struct MapDataEditor {
    pub asset: DataAssetEditor,
    editor: Editor,
    dialogs: Dialogs,
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            asset: DataAssetEditor::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, map_data: &mut MapData) {
        self.editor.map_editor.drop_selection(map_data);
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        self.dialogs.show(wc, map_data, tileset_ids, tilesets);

        let title = format!("{} - Map", map_data.asset.name);
        let window = DataAssetEditor::create_window(&mut self.asset, wc, &title);
        let (min_size, default_size) = calc_map_editor_window_size();
        window.min_size(min_size).default_size(default_size).show(wc.egui.ctx, |ui| {
            self.editor.show(ui, wc, &mut self.dialogs, map_data, tilesets);
        });
    }
}

struct Dialogs {
    properties_dialog: Option<PropertiesDialog>,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: None,
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, tileset_ids, tilesets);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    map_editor: MapEditorWidget,
    image_picker: ImagePickerWidget,
    use_custom_grid_color: bool,
    custom_grid_color: egui::Color32,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            map_editor: MapEditorWidget::new(),
            image_picker: ImagePickerWidget::new().use_as_palette(true),
            use_custom_grid_color: false,
            custom_grid_color: egui::Color32::RED,
        }
    }

    fn is_on_top(&self, wc: &WindowContext) -> bool {
        match wc.top_editor_asset_id {
            Some(top_id) => top_id == self.asset_id,
            None => false,
        }
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, dialogs: &mut Dialogs, map_data: &mut MapData) {
        egui::TopBottomPanel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Map", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            let dlg = dialogs.properties_dialog.get_or_insert_with(|| {
                                PropertiesDialog::new(map_data.tileset_id)
                            });
                            dlg.set_open(map_data, self.image_picker.selected_image_right as u8);
                        }
                    });
                });
                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Delete selection").clicked() {
                            self.map_editor.delete_selection(map_data);
                        }
                    });
                });
            });
        });
    }

    fn show_display_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext) {
        egui::TopBottomPanel::top(format!("editor_panel_{}_display_toolbar", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Display:");
                ui.add_space(1.0);

                if ui.add(egui::Button::image(IMAGES.layer_fg)
                          .selected(self.map_editor.display.has_bits(MapDisplay::FOREGROUND))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::FOREGROUND)))
                    .on_hover_text("Show foreground").clicked() {
                        self.map_editor.display.toggle(MapDisplay::FOREGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_bg)
                          .selected(self.map_editor.display.has_bits(MapDisplay::BACKGROUND))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::BACKGROUND)))
                    .on_hover_text("Show background").clicked() {
                        self.map_editor.display.toggle(MapDisplay::BACKGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_clip)
                          .selected(self.map_editor.display.has_bits(MapDisplay::CLIP))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::CLIP)))
                    .on_hover_text("Show collision").clicked() {
                        self.map_editor.display.toggle(MapDisplay::CLIP);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_fx)
                          .selected(self.map_editor.display.has_bits(MapDisplay::EFFECTS))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::EFFECTS)))
                    .on_hover_text("Show effects").clicked() {
                        self.map_editor.display.toggle(MapDisplay::EFFECTS);
                    }

                if ui.add(egui::Button::image(IMAGES.screen)
                          .selected(self.map_editor.display.has_bits(MapDisplay::SCREEN))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::SCREEN)))
                    .on_hover_text("Show screen size").clicked() {
                        self.map_editor.display.toggle(MapDisplay::SCREEN);
                    }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(egui::Button::image(IMAGES.grid)
                          .selected(self.map_editor.display.has_bits(MapDisplay::GRID))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::GRID)))
                    .on_hover_text("Show grid").clicked() {
                        self.map_editor.display.toggle(MapDisplay::GRID);
                    }

                if ui.add(egui::Button::new("Color").selected(self.use_custom_grid_color)).on_hover_text("Use custom grid color").clicked() {
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

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let cur_zoom_name = if let Some(zoom) = ZOOM_OPTIONS.iter().find(|z| **z == self.map_editor.zoom) {
                            &format!("{:3.1}x", zoom)
                        } else {
                            "custom"
                        };
                        egui::ComboBox::from_id_salt(format!("map_editor_{}_zoom_combo", self.asset_id))
                            .selected_text(cur_zoom_name)
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for zoom in ZOOM_OPTIONS {
                                    ui.selectable_value(&mut self.map_editor.zoom, *zoom, format!("{:3.1}x", zoom));
                                }
                            });
                        ui.add_space(1.0);
                        ui.label("Zoom:");
                    });
                });

                ui.spacing_mut().item_spacing = spacing;
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_edit_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, map_data: &mut MapData) {
        egui::TopBottomPanel::top(format!("editor_panel_{}_edit_toolbar", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                ui.label("Edit:");
                ui.add_space(23.0);

                if ui.add(egui::Button::image(IMAGES.pencil_fg)
                          .selected(self.map_editor.edit_layer == MapLayer::Foreground)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Foreground))
                    .on_hover_text("Edit foreground").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Foreground);
                        self.map_editor.display.set(MapDisplay::FOREGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.pencil_bg)
                          .selected(self.map_editor.edit_layer == MapLayer::Background)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Background))
                    .on_hover_text("Edit background").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Background);
                        self.map_editor.display.set(MapDisplay::BACKGROUND);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_clip)
                          .selected(self.map_editor.edit_layer == MapLayer::Clip)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Clip))
                    .on_hover_text("Edit collision").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Clip);
                        self.map_editor.display.set(MapDisplay::CLIP);
                    }

                if ui.add(egui::Button::image(IMAGES.layer_fx)
                          .selected(self.map_editor.edit_layer == MapLayer::Effects)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Effects))
                    .on_hover_text("Edit effects").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Effects);
                        self.map_editor.display.set(MapDisplay::EFFECTS);
                    }

                if ui.add(egui::Button::image(IMAGES.screen)
                          .selected(self.map_editor.edit_layer == MapLayer::Screen)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Screen))
                    .on_hover_text("Move screen size").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Screen);
                    }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
                ui.label("Tool:");
                ui.add_space(1.0);

                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.map_editor.tool == MapTool::Pencil)
                          .frame_when_inactive(self.map_editor.tool == MapTool::Pencil))
                    .on_hover_text("Place Tiles").clicked() {
                        self.map_editor.set_tool(MapTool::Pencil);
                    }

                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.map_editor.tool == MapTool::SelectLayer)
                          .frame_when_inactive(self.map_editor.tool == MapTool::SelectLayer))
                    .on_hover_text("Select current layer").clicked() {
                        self.map_editor.set_tool(MapTool::SelectLayer);
                    }

                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.map_editor.tool == MapTool::SelectFgLayers)
                          .frame_when_inactive(self.map_editor.tool == MapTool::SelectFgLayers))
                    .on_hover_text("Select foreground layers").clicked() {
                        self.map_editor.set_tool(MapTool::SelectFgLayers);
                    }

                if map_data.width == map_data.bg_width && map_data.height == map_data.bg_height &&
                    ui.add(egui::Button::image(IMAGES.select)
                           .selected(self.map_editor.tool == MapTool::SelectAllLayers)
                           .frame_when_inactive(self.map_editor.tool == MapTool::SelectAllLayers))
                    .on_hover_text("Select all layers").clicked() {
                        self.map_editor.set_tool(MapTool::SelectAllLayers);
                    }

                ui.spacing_mut().item_spacing = spacing;
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_footer(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, map_data: &mut MapData) {
        egui::TopBottomPanel::bottom(format!("editor_panel_{}_bottom", self.asset_id)).show_inside(ui, |ui| {
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                map_data: &mut MapData, tilesets: &AssetList<Tileset>) {
        self.show_menubar(ui, dialogs, map_data);
        self.show_display_toolbar(ui, wc);
        self.show_edit_toolbar(ui, wc, map_data);
        self.show_footer(ui, wc, map_data);

        if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
            // tile picker:
            egui::SidePanel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show_inside(ui, |ui| {
                ui.add_space(5.0);
                self.image_picker.zoom = 4.0;
                let (image, texture) = match self.map_editor.edit_layer {
                    MapLayer::Clip => {
                        ImageCollection::plus_static_texture(STATIC_IMAGES.clip_tiles(), wc.tex_man, wc.egui.ctx, TextureSlot::Transparent)
                    }
                    MapLayer::Effects => {
                        ImageCollection::plus_static_texture(STATIC_IMAGES.fx_tiles(), wc.tex_man, wc.egui.ctx, TextureSlot::Transparent)
                    }
                    _ => {
                        ImageCollection::plus_texture(tileset, wc.tex_man, wc.egui.ctx, self.image_picker.display.texture_slot())
                    }
                };
                self.image_picker.show(ui, wc.settings, &image, texture);
                self.map_editor.left_draw_tile = self.image_picker.selected_image;
                self.map_editor.right_draw_tile = self.image_picker.selected_image_right;
            });

            // body:
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.map_editor.show(ui, wc, map_data, tileset);
            });

            // keyboard:
            if self.is_on_top(wc) {
                self.map_editor.handle_keyboard(ui, map_data);
            }
        }
    }
}
