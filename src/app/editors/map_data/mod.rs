mod properties;

use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::data_asset::{
    MapData,
    Tileset,
    AssetIdList,
    AssetList,
    DataAssetId,
    GenericAsset,
};
use crate::misc::{
    IMAGES,
    STATIC_IMAGES,
};

use super::{
    AssetEditorBase,
    WindowContext,
    MapLayer,
    MapTileFixer,
};
use super::widgets::{
    MapEditorWidget,
    MapDisplay,
    MapTool,
    ImagePickerWidget,
};
use super::super::menu_item;

use properties::PropertiesDialog;

const ZOOM_OPTIONS: &[f32] = &[ 0.5, 0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0 ];

pub struct MapDataEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl MapDataEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        MapDataEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, map_data: &mut MapData) {
        self.editor.map_editor.drop_selection(map_data);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, editor: &Editor, map_data: &MapData, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, map_data.asset.id));
        let dirty = if base.is_dirty() { " (modified)" } else { "" };
        egui::Panel::bottom(format!("editor_panel_{}_bottom", map_data.asset.id)).frame(bottom_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(format!(
                    "{} bytes [size: {}x{}, parallax: {}x{}]{}",
                    map_data.data_size(),
                    map_data.width,
                    map_data.height,
                    map_data.para_width,
                    map_data.para_height,
                    dirty
                )).truncate());
                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        ui.add_space(1.0);
                        ui.label(format!("({}, {})", editor.map_editor.hover_pos.x.floor(), editor.map_editor.hover_pos.y.floor()));
                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        self.dialogs.show(wc, &mut self.editor, map_data, tileset_ids, tilesets);

        let min_size = egui::Vec2::new(600.0, 200.0);
        let def_size = egui::Vec2::new(map_data.width as f32, map_data.height as f32) * Tileset::TILE_SIZE as f32;
        let def_size = def_size.min(wc.window_space.size() - egui::Vec2::splat(100.0)).max(min_size);

        self.base.show_window(wc, map_data, min_size, def_size, |ui, wc, map_data, base| {
            Self::show_footer(ui, wc, &self.editor, map_data, base);
            self.editor.show(ui, wc, &mut self.dialogs, map_data, tilesets);
        });
    }
}

impl MapTileFixer for MapDataEditor {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]> {
        self.editor.map_editor.get_tile_planes()
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

    pub fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor,
                map_data: &mut MapData, tileset_ids: &AssetIdList, tilesets: &AssetList<Tileset>) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, tileset_ids, tilesets);
            if dlg.resized || dlg.changed_tileset {
                editor.map_editor.set_undo_target(map_data);
            }
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    map_editor: MapEditorWidget,
    image_picker: ImagePickerWidget,
    use_custom_grid_color: bool,
    use_custom_bg_color: bool,
    custom_grid_color: egui::Color32,
    custom_bg_color: egui::Color32,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            map_editor: MapEditorWidget::new(),
            image_picker: ImagePickerWidget::new().use_as_palette(true),
            use_custom_grid_color: false,
            use_custom_bg_color: false,
            custom_grid_color: egui::Color32::RED,
            custom_bg_color: egui::Color32::from_rgb(0, 0xffu8, 0),
        }
    }

    fn tile_to_image_selection(tile: u8) -> Option<u32> {
        if tile == MapData::NO_TILE {
            None
        } else {
            Some(tile as u32)
        }
    }

    fn image_selection_to_tile(image_selection: Option<u32>) -> u8 {
        if let Some(image_selection) = image_selection {
            (image_selection & 0xff) as u8
        } else {
            MapData::NO_TILE
        }
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, map_data: &mut MapData) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Map", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        let dlg = dialogs.properties_dialog.get_or_insert_with(|| {
                            PropertiesDialog::new(map_data.tileset_id)
                        });
                        dlg.set_open(wc, map_data, Self::image_selection_to_tile(self.image_picker.get_selected_image_right()));
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(self.map_editor.can_undo(), menu_item(IMAGES.undo, " Undo")).clicked() {
                        self.map_editor.undo(map_data);
                    }

                    ui.separator();

                    let has_selection = ! self.map_editor.selection.is_empty();
                    if ui.add_enabled(has_selection, menu_item(IMAGES.cut, " Cut")).clicked() {
                        self.map_editor.cut(wc, map_data);
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.copy, " Copy")).clicked() {
                        self.map_editor.copy(wc, map_data);
                    }
                    if ui.add_enabled(! wc.map_clipboard.is_none(), menu_item(IMAGES.paste, " Paste")).clicked() {
                        self.map_editor.paste(wc, map_data);
                    }
                    if ui.add_enabled(has_selection, menu_item(IMAGES.trash, " Delete selection")).clicked() {
                        self.map_editor.delete_selection(map_data);
                    }
                });
            });
        });
    }

    fn add_indenting_label(ui: &mut egui::Ui, width: f32, text: &str) {
        let start = ui.cursor();
        ui.label(text);
        let end = ui.cursor();
        let space_left = width - (end.min.x - start.min.x);
        if space_left > 0.0 {
            let indent = egui::Rect::from_min_size(end.min, egui::Vec2::new(space_left, 1.0));
            ui.advance_cursor_after_rect(indent);
        }
    }

    fn show_display_toolbar(&mut self, ui: &mut egui::Ui, _wc: &mut WindowContext, map_data: &MapData) {
        egui::Panel::top(format!("editor_panel_{}_display_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                Self::add_indenting_label(ui, 70.0, "Display:");

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

                if ui.add(egui::Button::image(IMAGES.layer_fx)
                          .selected(self.map_editor.display.has_bits(MapDisplay::EFFECTS))
                          .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::EFFECTS)))
                    .on_hover_text("Show effects").clicked() {
                        self.map_editor.display.toggle(MapDisplay::EFFECTS);
                    }

                if ui.add_enabled(map_data.para_width != 0 && map_data.para_height != 0,
                                  egui::Button::image(IMAGES.layer_parallax)
                                  .selected(self.map_editor.display.has_bits(MapDisplay::PARALLAX))
                                  .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::PARALLAX)))
                    .on_hover_text("Show parallax").clicked() {
                        self.map_editor.display.toggle(MapDisplay::PARALLAX);
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
                    ui.add(egui::Label::new("default").selectable(false));
                    self.map_editor.custom_grid_color = None;
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(egui::Button::new("BG Color").selected(self.use_custom_bg_color))
                    .on_hover_text("Use custom background color").clicked() {
                        self.use_custom_bg_color = ! self.use_custom_bg_color;
                    }
                ui.add_space(2.0);
                if self.use_custom_bg_color {
                    let mut rgba = self.custom_bg_color.into();
                    egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                    self.custom_bg_color = rgba.into();
                    self.map_editor.custom_bg_color = Some(self.custom_bg_color);
                } else {
                    ui.add_space(2.0);
                    ui.add(egui::Label::new("default").selectable(false));
                    self.map_editor.custom_bg_color = None;
                }

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let cur_zoom_name = if let Some(zoom) = ZOOM_OPTIONS.iter().find(|&z| *z == self.map_editor.zoom) {
                            &format!("{:3.2}x", zoom)
                        } else {
                            "custom"
                        };
                        egui::ComboBox::from_id_salt(format!("map_editor_{}_zoom_combo", self.asset_id))
                            .selected_text(cur_zoom_name)
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for zoom in ZOOM_OPTIONS {
                                    ui.selectable_value(&mut self.map_editor.zoom, *zoom, format!("{:3.2}x", zoom));
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
        egui::Panel::top(format!("editor_panel_{}_edit_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                Self::add_indenting_label(ui, 70.0, "Edit:");

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

                if ui.add(egui::Button::image(IMAGES.layer_fx)
                          .selected(self.map_editor.edit_layer == MapLayer::Effects)
                          .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Effects))
                    .on_hover_text("Edit effects").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Effects);
                        self.map_editor.display.set(MapDisplay::EFFECTS);
                    }

                if ui.add_enabled(map_data.para_width != 0 && map_data.para_height != 0,
                                  egui::Button::image(IMAGES.layer_parallax)
                                  .selected(self.map_editor.edit_layer == MapLayer::Parallax)
                                  .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Parallax))
                    .on_hover_text("Edit parallax").clicked() {
                        self.map_editor.set_edit_layer(MapLayer::Parallax);
                        self.map_editor.display.set(MapDisplay::PARALLAX);
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

                let tools_enabled = self.map_editor.edit_layer != MapLayer::Screen;

                if ui.add_enabled(tools_enabled, egui::Button::image(IMAGES.pen)
                                  .selected(self.map_editor.tool == MapTool::Pencil)
                                  .frame_when_inactive(self.map_editor.tool == MapTool::Pencil))
                    .on_hover_text("Place Tiles").clicked() {
                        self.map_editor.set_tool(MapTool::Pencil);
                    }

                if ui.add_enabled(tools_enabled, egui::Button::image(IMAGES.select)
                                  .selected(self.map_editor.tool == MapTool::SelectLayer)
                                  .frame_when_inactive(self.map_editor.tool == MapTool::SelectLayer))
                    .on_hover_text("Select current layer").clicked() {
                        self.map_editor.set_tool(MapTool::SelectLayer);
                    }

                if ui.add_enabled(tools_enabled, egui::Button::image(IMAGES.select)
                                  .selected(self.map_editor.tool == MapTool::SelectFullLayers)
                                  .frame_when_inactive(self.map_editor.tool == MapTool::SelectFullLayers))
                    .on_hover_text("Select normal layers").clicked() {
                            self.map_editor.set_tool(MapTool::SelectFullLayers);
                    }

                if map_data.width == map_data.para_width && map_data.height == map_data.para_height &&
                    ui.add_enabled(tools_enabled, egui::Button::image(IMAGES.select)
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

    pub fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs,
                map_data: &mut MapData, tilesets: &AssetList<Tileset>) {
        self.show_menubar(ui, wc, dialogs, map_data);
        self.show_display_toolbar(ui, wc, map_data);
        self.show_edit_toolbar(ui, wc, map_data);

        if let Some(tileset) = tilesets.get(&map_data.tileset_id) {
            // tile picker:
            egui::Panel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show(ui, |ui| {
                ui.add_space(5.0);
                self.image_picker.zoom = 4.0;
                match self.map_editor.edit_layer {
                    MapLayer::Effects => {
                        let tiles = STATIC_IMAGES.fx_tiles();
                        let texture = tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                        self.image_picker.show(ui, wc.settings, tiles, texture, egui::Color32::BLACK);
                    }
                    _ => {
                        let bg_color = if self.use_custom_bg_color { self.custom_bg_color } else { wc.settings.map_bg_color };
                        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                        self.image_picker.show(ui, wc.settings, tileset, texture, bg_color);
                    }
                }
                self.map_editor.left_draw_tile = Self::image_selection_to_tile(self.image_picker.get_selected_image());
                self.map_editor.right_draw_tile = Self::image_selection_to_tile(self.image_picker.get_selected_image_right());
            });

            // body:
            egui::CentralPanel::default().show(ui, |ui| {
                self.map_editor.show(ui, wc, map_data, tileset);
                if self.map_editor.left_draw_tile_changed {
                    self.map_editor.left_draw_tile_changed = false;
                    self.image_picker.set_selected_image(Self::tile_to_image_selection(self.map_editor.left_draw_tile));
                }
                if self.map_editor.right_draw_tile_changed {
                    self.map_editor.right_draw_tile_changed = false;
                    self.image_picker.set_selected_image_right(Self::tile_to_image_selection(self.map_editor.right_draw_tile));
                }
            });

            // keyboard:
            if wc.is_editor_on_top(self.asset_id) {
                self.map_editor.handle_keyboard(ui, wc, map_data);
            }
        }
    }
}
