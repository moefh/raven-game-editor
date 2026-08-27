mod properties;
mod custom_colors;

use core::fmt::NumBuffer;

use crate::image::{
    ImageCollection,
    TextureSlot,
};
use crate::data_asset::{
    self,
    AssetList,
    AssetIdCollection,
    DataAssetId,
    GenericAsset,
    MapData,
    Tileset,
    TileAnimation,
};
use crate::misc::{
    IMAGES,
    STATIC_IMAGES,
};

use super::{
    AssetEditorBase,
    WindowContext,
    SysDialogResponse,
    SysDialogOpenFile,
    MapLayer,
    MapTileFixer,
    EditorAction,
};
use super::widgets::{
    MapEditorWidget,
    MapDisplay,
    MapTool,
    ImagePickerWidget,
    TilePickerPopupWidget,
};
use super::super::{
    menu_item,
    menu_item_no_image,
};

use properties::PropertiesDialog;
use custom_colors::CustomColorsDialog;

const ZOOM_OPTIONS: &[f32] = &[ 0.5, 0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0 ];

pub struct MapDataEditorAssetLists<'a> {
    pub tilesets: &'a AssetList<Tileset>,
    pub tile_anims: &'a AssetList<TileAnimation>,
}

impl<'a> MapDataEditorAssetLists<'a> {
    pub fn new(
        tilesets: &'a AssetList<Tileset>,
        tile_anims: &'a AssetList<TileAnimation>,
    ) -> Self {
        MapDataEditorAssetLists {
            tilesets,
            tile_anims,
        }
    }
}

pub struct CustomColors {
    pub use_grid_color: bool,
    pub use_bg_color: bool,
    pub grid_color: egui::Color32,
    pub bg_color: egui::Color32,
}

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
                        ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 0.0);

                        ui.add_space(1.0);

                        ui.label(format!("({}, {})", editor.map_editor.hover_pos.x.floor(), editor.map_editor.hover_pos.y.floor()));

                        if let Some(sel_rect) = editor.get_selection_rectangle() && sel_rect.is_positive() {
                            ui.label(format!("[sel {}x{}]", sel_rect.width(), sel_rect.height()));
                        }

                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
        });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        map_data: &mut MapData,
        asset_ids: &AssetIdCollection,
        assets: &MapDataEditorAssetLists,
    ) {
        self.dialogs.show(wc, &mut self.editor, map_data, asset_ids, assets.tilesets, assets.tile_anims);

        let min_size = egui::Vec2::new(670.0, 200.0);
        let def_size = egui::Vec2::new(map_data.width as f32, map_data.height as f32 * 1.2) * Tileset::TILE_SIZE as f32;
        let def_size = def_size.min(wc.window_space.size() - egui::Vec2::splat(100.0)).max(min_size);

        self.base.show_window(wc, map_data, min_size, def_size, |ui, wc, map_data, base| {
            Self::show_footer(ui, wc, &self.editor, map_data, base);
            self.editor.show(ui, wc, &mut self.dialogs, map_data, asset_ids, assets);
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
    custom_colors_dialog: CustomColorsDialog,
}

impl Dialogs {
    fn new() -> Self {
        Dialogs {
            properties_dialog: None,
            custom_colors_dialog: CustomColorsDialog::new(),
        }
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        editor: &mut Editor,
        map_data: &mut MapData,
        asset_ids: &AssetIdCollection,
        tilesets: &AssetList<Tileset>,
        tile_anims: &AssetList<TileAnimation>,
    ) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, map_data, asset_ids, tilesets, tile_anims);
            if dlg.resized || dlg.changed_tileset {
                editor.map_editor.set_undo_target(map_data);
            }
        }

        if self.custom_colors_dialog.open && self.custom_colors_dialog.show(wc, &mut editor.custom_colors) {
            editor.map_editor.custom_grid_color = if editor.custom_colors.use_grid_color {
                Some(editor.custom_colors.grid_color)
            } else {
                None
            };
            editor.map_editor.custom_bg_color = if editor.custom_colors.use_bg_color {
                Some(editor.custom_colors.bg_color)
            } else {
                None
            };
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    import_sys_dlg_id: String,
    map_editor: MapEditorWidget,
    image_picker: ImagePickerWidget,
    tile_picker_popup: TilePickerPopupWidget,
    custom_colors: CustomColors,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            import_sys_dlg_id: format!("editor_{}_import_map", asset_id),
            map_editor: MapEditorWidget::new(),
            image_picker: ImagePickerWidget::new().use_as_palette(true),
            tile_picker_popup: TilePickerPopupWidget::new(egui::Id::new(format!("editor_{}_tile_picker_popup", asset_id)), true),
            custom_colors: CustomColors {
                use_grid_color: false,
                use_bg_color: false,
                grid_color: egui::Color32::RED,
                bg_color: egui::Color32::from_rgb(0, 0xffu8, 0),
            },
        }
    }

    fn get_selection_rectangle(&self) -> Option<egui::Rect> {
        self.map_editor.selection.get_rect()
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

    fn get_picker_tile_name(tile: Option<u32>, buf: &mut NumBuffer<u32>) -> &str {
        if let Some(tile) = tile {
            tile.format_into(buf)
        } else {
            "-"
        }
     }

    fn get_map_tile_name(tile: Option<u8>, buf: &mut NumBuffer<u8>) -> &str {
        if let Some(tile) = tile && tile != MapData::NO_TILE {
            tile.format_into(buf)
        } else {
            "-"
        }
    }

    fn show_menubar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, map_data: &mut MapData) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Map", |ui| {
                        if ui.add(menu_item(IMAGES.import, " Import...")).clicked() {
                            wc.sys_dialogs.open_file(
                                Some(wc.egui.window),
                                self.import_sys_dlg_id.clone(),
                                "map",
                                "Import Map file",
                                &[
                                    ("Raven Map files (*.ravmap)", &["ravmap"]),
                                    ("All files (*.*)", &["*"]),
                                ]
                            );
                        }

                        if ui.add(menu_item(IMAGES.export, " Export...")).clicked() {
                            wc.add_editor_action(EditorAction::ExportMap { map_id: self.asset_id });
                        }

                        ui.separator();

                        if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                            let dlg = dialogs.properties_dialog.get_or_insert_with(|| {
                                PropertiesDialog::new(map_data.tileset_id)
                            });
                            dlg.set_open(wc, map_data, Self::image_selection_to_tile(self.image_picker.get_selected_image_r()));
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.add_enabled(self.map_editor.can_undo(), menu_item(IMAGES.undo, " Undo")).clicked() {
                            self.map_editor.undo(map_data);
                        }
                        if ui.add_enabled(self.map_editor.can_redo(), menu_item(IMAGES.redo, " Redo")).clicked() {
                            self.map_editor.redo(map_data);
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
                    ui.menu_button("View", |ui| {
                        if ui.add(menu_item_no_image(" Custom Colors")).clicked() {
                            dialogs.custom_colors_dialog.set_open(wc);
                        }
                    });
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

    fn show_display_toolbar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, map_data: &MapData) {
        egui::Panel::top(format!("editor_panel_{}_display_toolbar", self.asset_id)).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                Self::add_indenting_label(ui, 70.0, "Display:");

                if ui.add(
                    egui::Button::image(IMAGES.layer_fg)
                        .selected(self.map_editor.display.has_bits(MapDisplay::FOREGROUND))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::FOREGROUND))
                ).on_hover_text("Show foreground").clicked() {
                    self.map_editor.display.toggle(MapDisplay::FOREGROUND);
                }

                if ui.add(
                    egui::Button::image(IMAGES.layer_bg)
                        .selected(self.map_editor.display.has_bits(MapDisplay::BACKGROUND))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::BACKGROUND))
                ).on_hover_text("Show background").clicked() {
                    self.map_editor.display.toggle(MapDisplay::BACKGROUND);
                }

                if ui.add(
                    egui::Button::image(IMAGES.layer_fx)
                        .selected(self.map_editor.display.has_bits(MapDisplay::EFFECTS))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::EFFECTS))
                ).on_hover_text("Show effects").clicked() {
                    self.map_editor.display.toggle(MapDisplay::EFFECTS);
                }

                if ui.add(
                    egui::Button::image(IMAGES.layer_anim)
                        .selected(self.map_editor.display.has_bits(MapDisplay::ANIMATION))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::ANIMATION))
                ).on_hover_text("Show animation").clicked() {
                    self.map_editor.display.toggle(MapDisplay::ANIMATION);
                }

                if ui.add_enabled(
                    map_data.para_width != 0 && map_data.para_height != 0,
                    egui::Button::image(IMAGES.layer_parallax)
                        .selected(self.map_editor.display.has_bits(MapDisplay::PARALLAX))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::PARALLAX))
                ).on_hover_text("Show parallax").clicked() {
                    self.map_editor.display.toggle(MapDisplay::PARALLAX);
                }

                if ui.add(
                    egui::Button::image(IMAGES.screen)
                        .selected(self.map_editor.display.has_bits(MapDisplay::SCREEN))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::SCREEN))
                ).on_hover_text("Show screen size").clicked() {
                    self.map_editor.display.toggle(MapDisplay::SCREEN);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(
                    egui::Button::image(IMAGES.tile_anim)
                        .selected(self.map_editor.display.has_bits(MapDisplay::ANIMATE_TILES))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::ANIMATE_TILES))
                ).on_hover_text("Animate tiles").clicked() {
                    self.map_editor.display.toggle(MapDisplay::ANIMATE_TILES);
                }

                ui.add_space(5.0);

                if ui.add(
                    egui::Button::image(IMAGES.grid)
                        .selected(self.map_editor.display.has_bits(MapDisplay::GRID))
                        .frame_when_inactive(self.map_editor.display.has_bits(MapDisplay::GRID))
                ).on_hover_text("Show grid").clicked() {
                    self.map_editor.display.toggle(MapDisplay::GRID);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.button("Colors").on_hover_text("Use custom colors").clicked() {
                    dialogs.custom_colors_dialog.set_open(wc);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.label("Zoom:");
                ui.add_space(1.0);
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

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 0.0);

                        ui.add_space(5.0);

                        let mut fg_tile_buf = NumBuffer::new();
                        let mut bg_tile_buf = NumBuffer::new();
                        let mut fx_tile_buf = NumBuffer::new();
                        let mut an_tile_buf = NumBuffer::new();

                        let fg_tile = Self::get_map_tile_name(self.map_editor.hover_tile_fg, &mut fg_tile_buf);
                        let bg_tile = Self::get_map_tile_name(self.map_editor.hover_tile_bg, &mut bg_tile_buf);
                        let fx_tile = Self::get_map_tile_name(self.map_editor.hover_tile_fx, &mut fx_tile_buf);
                        let an_tile = Self::get_map_tile_name(self.map_editor.hover_tile_an, &mut an_tile_buf);

                        ui.label(format!(
                            "[tile {}/{}/{}/{}]",
                            fg_tile,
                            bg_tile,
                            fx_tile,
                            an_tile
                        ));

                        ui.spacing_mut().item_spacing = spacing;
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

                if ui.add(
                    egui::Button::image(IMAGES.pencil_fg)
                        .selected(self.map_editor.edit_layer == MapLayer::Foreground)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Foreground)
                ).on_hover_text("Edit foreground").clicked() {
                    self.map_editor.set_edit_layer(MapLayer::Foreground);
                    self.map_editor.display.set(MapDisplay::FOREGROUND);
                }

                if ui.add(
                    egui::Button::image(IMAGES.pencil_bg)
                        .selected(self.map_editor.edit_layer == MapLayer::Background)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Background)
                ).on_hover_text("Edit background").clicked() {
                    self.map_editor.set_edit_layer(MapLayer::Background);
                    self.map_editor.display.set(MapDisplay::BACKGROUND);
                }

                if ui.add(
                    egui::Button::image(IMAGES.layer_fx)
                        .selected(self.map_editor.edit_layer == MapLayer::Effects)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Effects)
                ).on_hover_text("Edit effects").clicked() {
                    self.map_editor.set_edit_layer(MapLayer::Effects);
                    self.map_editor.display.set(MapDisplay::EFFECTS);
                }

                if ui.add(
                    egui::Button::image(IMAGES.layer_anim)
                        .selected(self.map_editor.edit_layer == MapLayer::Animation)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Animation)
                ).on_hover_text("Edit animation").clicked() {
                    self.map_editor.edit_layer = MapLayer::Animation;
                    self.map_editor.display.set(MapDisplay::ANIMATION);
                }

                if ui.add_enabled(
                    map_data.para_width != 0 && map_data.para_height != 0,
                    egui::Button::image(IMAGES.layer_parallax)
                        .selected(self.map_editor.edit_layer == MapLayer::Parallax)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Parallax)
                ).on_hover_text("Edit parallax").clicked() {
                    self.map_editor.set_edit_layer(MapLayer::Parallax);
                    self.map_editor.display.set(MapDisplay::PARALLAX);
                }

                if ui.add(
                    egui::Button::image(IMAGES.screen)
                        .selected(self.map_editor.edit_layer == MapLayer::Screen)
                        .frame_when_inactive(self.map_editor.edit_layer == MapLayer::Screen)
                ).on_hover_text("Move screen size").clicked() {
                    self.map_editor.set_edit_layer(MapLayer::Screen);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
                ui.label("Tool:");
                ui.add_space(1.0);

                let tools_enabled = self.map_editor.edit_layer != MapLayer::Screen;

                if ui.add_enabled(
                    tools_enabled,
                    egui::Button::image(IMAGES.pen)
                        .selected(self.map_editor.tool == MapTool::Pencil)
                        .frame_when_inactive(self.map_editor.tool == MapTool::Pencil)
                ).on_hover_text("Place Tiles").clicked() {
                    self.map_editor.set_tool(MapTool::Pencil);
                }

                if ui.add_enabled(
                    tools_enabled,
                    egui::Button::image(IMAGES.select)
                        .selected(self.map_editor.tool == MapTool::SelectLayer)
                        .frame_when_inactive(self.map_editor.tool == MapTool::SelectLayer)
                ).on_hover_text("Select current layer").clicked() {
                    self.map_editor.set_tool(MapTool::SelectLayer);
                }

                if ui.add_enabled(
                    tools_enabled,
                    egui::Button::image(IMAGES.select)
                        .selected(self.map_editor.tool == MapTool::SelectFullLayers)
                        .frame_when_inactive(self.map_editor.tool == MapTool::SelectFullLayers)
                ).on_hover_text("Select normal layers").clicked() {
                        self.map_editor.set_tool(MapTool::SelectFullLayers);
                    }

                if map_data.width == map_data.para_width && map_data.height == map_data.para_height &&
                    ui.add_enabled(
                        tools_enabled,
                        egui::Button::image(IMAGES.select)
                            .selected(self.map_editor.tool == MapTool::SelectAllLayers)
                            .frame_when_inactive(self.map_editor.tool == MapTool::SelectAllLayers)
                    ).on_hover_text("Select all layers").clicked() {
                        self.map_editor.set_tool(MapTool::SelectAllLayers);
                    }

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let mut left_buf = NumBuffer::new();
                        let mut right_buf = NumBuffer::new();
                        let sel_set = match self.map_editor.edit_layer {
                            MapLayer::Foreground | MapLayer::Background => { 0 }
                            MapLayer::Effects => { 1 }
                            MapLayer::Animation => { 2 }
                            _ => { 0 }
                        };
                        let left_name = Self::get_picker_tile_name(self.image_picker.get_selected_image_l_for_set(sel_set), &mut left_buf);
                        let right_name = Self::get_picker_tile_name(self.image_picker.get_selected_image_r_for_set(sel_set), &mut right_buf);
                        ui.label(format!("[tile {}/{}]", left_name, right_name));
                    });
                });

                ui.spacing_mut().item_spacing = spacing;
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn import_map(&mut self, wc: &mut WindowContext, file: SysDialogOpenFile, map_data: &mut MapData, asset_ids: &AssetIdCollection) {
        let result = file.read_string().and_then(|content| {
            data_asset::deserialize_map(&content, self.asset_id, asset_ids, wc.logger)
        });
        match result {
            Ok(mut new_map_data) => {
                std::mem::swap(map_data, &mut new_map_data)
            }

            Err(e) => {
                wc.logger.log(format!("ERROR reading map file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box("Error importing Map", "Error importing map file.\n\nConsult the log window for more information.");
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        map_data: &mut MapData,
        asset_ids: &AssetIdCollection,
        assets: &MapDataEditorAssetLists,
    ) {
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sys_dlg_id) {
            self.import_map(wc, file, map_data, asset_ids);
        }

        self.show_menubar(ui, wc, dialogs, map_data);
        self.show_display_toolbar(ui, wc, dialogs, map_data);
        self.show_edit_toolbar(ui, wc, map_data);

        // tile picker:
        egui::Panel::left(format!("editor_panel_{}_left", self.asset_id)).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            self.image_picker.zoom = 4.0;
            match self.map_editor.edit_layer {
                MapLayer::Effects => {
                    let tiles = STATIC_IMAGES.fx_tiles();
                    let texture = tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                    if self.image_picker.selection_set != 1 {
                        self.image_picker.selection_set = 1;
                        self.image_picker.force_selection_into_visibility();
                    }
                    self.image_picker.show(ui, wc.settings, tiles, texture, egui::Color32::BLACK);
                }
                MapLayer::Animation => {
                    let tiles = STATIC_IMAGES.anim_tiles();
                    let texture = tiles.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                    if self.image_picker.selection_set != 2 {
                        self.image_picker.selection_set = 2;
                        self.image_picker.force_selection_into_visibility();
                    }
                    self.image_picker.show(ui, wc.settings, tiles, texture, egui::Color32::BLACK);
                }
                _ => {
                    if let Some(tileset) = assets.tilesets.get(&map_data.tileset_id) {
                        let color_picker_response = ui.button("Select");
                        if color_picker_response.clicked() {
                            self.tile_picker_popup.open();
                        }
                        let mut tile = self.image_picker.get_selected_image_l().map(|tile| tile as u8);
                        if self.tile_picker_popup.show(wc, &color_picker_response, tileset, &mut tile) {
                            self.image_picker.set_selected_image_l(tile.map(|tile| tile as u32));
                        }

                        let bg_color = if self.custom_colors.use_bg_color { self.custom_colors.bg_color } else { wc.settings.map_bg_color };
                        let texture = tileset.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                        if self.image_picker.selection_set != 0 {
                            self.image_picker.selection_set = 0;
                            self.image_picker.force_selection_into_visibility();
                        }
                        self.image_picker.show(ui, wc.settings, tileset, texture, bg_color);
                    }
                }
            }
            self.map_editor.left_draw_tile = Self::image_selection_to_tile(self.image_picker.get_selected_image_l());
            self.map_editor.right_draw_tile = Self::image_selection_to_tile(self.image_picker.get_selected_image_r());
        });

        // body:
        egui::CentralPanel::default().show(ui, |ui| {
            self.map_editor.show(ui, wc, map_data, assets.tilesets, assets.tile_anims);
            if self.map_editor.left_draw_tile_changed {
                self.map_editor.left_draw_tile_changed = false;
                self.image_picker.set_selected_image_l(Self::tile_to_image_selection(self.map_editor.left_draw_tile));
            }
            if self.map_editor.right_draw_tile_changed {
                self.map_editor.right_draw_tile_changed = false;
                self.image_picker.set_selected_image_r(Self::tile_to_image_selection(self.map_editor.right_draw_tile));
            }
        });

        // keyboard:
        if wc.is_editor_on_top(self.asset_id) {
            self.map_editor.handle_keyboard(ui, wc, map_data);
        }
    }
}
