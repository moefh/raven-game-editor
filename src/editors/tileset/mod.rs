mod properties;
mod add_tiles;
mod remove_tiles;
mod import;
mod export;
mod create_colorset;

use crate::misc::IMAGES;
use crate::app::{WindowContext, SysDialogResponse};
use crate::image::{colors, ImageCollection, ImagePixels, ImageRotation};
use crate::data_asset::{
    DataAssetId,
    GenericAsset,
    Tileset,
};

use properties::PropertiesDialog;
use remove_tiles::RemoveTilesDialog;
use add_tiles::{AddTilesDialog, AddTilesAction};
use export::ExportDialog;
use import::ImportDialog;
use create_colorset::CreateColorsetDialog;
use super::{
    AssetEditorBase,
    TileGrid,
    TileGridImage,
    MapTileFixer,
};
use super::widgets::{
    ColorPickerWidget,
    ColorPickerResponse,
    ImagePickerWidget,
    ImageEditorWidget,
    TileGridEditorWidget,
    TileGridEditorAction,
    ImageEditorAction,
    ImageDrawingTool,
    ImageDisplay,
};

enum EditorTab {
    Tile,
    Grid,
    GridTiles,
}

pub struct TilesetEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl TilesetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TilesetEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(format!("editor_{}", id)),
        }
    }

    pub fn prepare_for_saving(&mut self, tileset: &mut Tileset) {
        self.editor.prepare_for_saving(tileset);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, editor: &Editor, base: &AssetEditorBase, tileset: &Tileset) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, tileset.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", tileset.asset.id)).frame(bottom_frame).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty = if base.is_dirty() { " (modified)" } else { "" };
                ui.label(format!("{} bytes [{} tiles]{}", tileset.data_size(), tileset.num_tiles, dirty));

                if let Some(tile) = editor.tile_picker.get_selected_image() {
                    ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let spacing = ui.spacing().item_spacing;
                            ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                            ui.add_space(1.0);
                            ui.label(format!("tile {}", tile));
                            ui.spacing_mut().item_spacing = spacing;
                        });
                    });
                }
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        self.dialogs.show(wc, &mut self.editor, tileset);

        let (min_size, default_size) = AssetEditorBase::calc_image_editor_window_size(tileset);
        let min_size = min_size.max(egui::Vec2::new(500.0, 400.0));
        let default_size = default_size.max(egui::Vec2::new(500.0, 400.0));
        self.base.show_window(wc, tileset, min_size, default_size, |ui, wc, tileset, base| {
            Self::show_footer(ui, wc, &self.editor, base, tileset);
            self.editor.show(ui, wc, &mut self.dialogs, tileset);
        });
    }
}

impl MapTileFixer for TilesetEditor {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]> {
        self.editor.tile_grid.get_tile_planes_mut()
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    add_tiles_dialog: AddTilesDialog,
    rm_tiles_dialog: RemoveTilesDialog,
    import_dialog: ImportDialog,
    export_dialog: ExportDialog,
    create_colorset_dialog: CreateColorsetDialog,
}

impl Dialogs {
    fn new(id_prefix: impl AsRef<str>) -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            add_tiles_dialog: AddTilesDialog::new(),
            rm_tiles_dialog: RemoveTilesDialog::new(),
            import_dialog: ImportDialog::new(),
            export_dialog: ExportDialog::new(),
            create_colorset_dialog: CreateColorsetDialog::new(id_prefix),
       }
    }

    fn ensure_valid_selected_image(&self, editor: &mut Editor, tileset: &Tileset, set_undo_target: bool) {
        if editor.tile_image_editor.get_selected_image() >= tileset.num_tiles {
            let selected_image = tileset.num_tiles - 1;
            editor.tile_picker.set_selected_image(Some(selected_image));
            let no_selection_change = ! editor.tile_image_editor.set_selected_image(selected_image, tileset);
            if no_selection_change && set_undo_target {
                editor.tile_image_editor.set_undo_target(tileset);
            }
        }
    }

    fn show(&mut self, wc: &mut WindowContext, editor: &mut Editor, tileset: &mut Tileset) {
        if self.properties_dialog.open && self.properties_dialog.show(wc, tileset) {
            self.ensure_valid_selected_image(editor, tileset, false);
            editor.tile_image_editor.set_undo_target(tileset);
            editor.tile_image_editor.set_image_changed();
        }
        if self.add_tiles_dialog.open && self.add_tiles_dialog.show(wc, tileset) {
            editor.tile_image_editor.set_undo_target(tileset);
            editor.tile_image_editor.set_image_changed();
        }
        if self.rm_tiles_dialog.open && self.rm_tiles_dialog.show(wc, tileset) {
            self.ensure_valid_selected_image(editor, tileset, false);
            editor.tile_image_editor.set_undo_target(tileset);
            editor.tile_image_editor.set_image_changed();
        }
        if self.export_dialog.open {
            self.export_dialog.show(wc, tileset);
            editor.tile_image_editor.set_image_changed();
        }
        if self.import_dialog.open && self.import_dialog.show(wc, tileset) {
            self.ensure_valid_selected_image(editor, tileset, true);
            editor.tile_image_editor.set_image_changed();
        }
        if self.create_colorset_dialog.open && self.create_colorset_dialog.show(wc, tileset) {
            editor.color_picker.set_colorset(self.create_colorset_dialog.created_colorset_index);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    tile_picker_panel_id: egui::Id,
    selected_tab: EditorTab,
    color_picker: ColorPickerWidget,
    tile_picker: ImagePickerWidget,
    tile_image_editor: ImageEditorWidget<Tileset>,
    grid_tile_picker: ImagePickerWidget,
    grid_image_editor: ImageEditorWidget<TileGridImage>,
    tile_grid_editor: TileGridEditorWidget,
    tile_grid: TileGrid,
}

impl Editor {
    const MAX_TILE_GRID_WIDTH: u32 = 10;
    const MAX_TILE_GRID_HEIGHT: u32 = 10;

    fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            tile_picker_panel_id: egui::Id::new(format!("editor_panel_{}_left", asset_id)),
            selected_tab: EditorTab::Tile,
            color_picker: ColorPickerWidget::new(format!("editor_{}_color_picker", asset_id), colors::RED, colors::BLUE, true),
            tile_picker: ImagePickerWidget::new(),
            tile_image_editor: ImageEditorWidget::new(),
            grid_tile_picker: ImagePickerWidget::new().use_as_palette(true),
            grid_image_editor: ImageEditorWidget::new(),
            tile_grid_editor: TileGridEditorWidget::new(),
            tile_grid: TileGrid::new(asset_id),
        }
    }

    fn prepare_for_saving(&mut self, tileset: &mut Tileset) {
        self.tile_image_editor.drop_selection(tileset);

        let grid_image = self.tile_grid.get_image_mut(tileset);
        self.grid_image_editor.drop_selection(grid_image);
        self.tile_grid.image_to_tileset(tileset);
    }

    fn handle_grid_image_changed(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        self.tile_grid.image_to_tileset(tileset);
        ImageEditorWidget::<Tileset>::update_texture(wc, tileset);
    }

    fn vflip(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.vflip(tileset, self.color_picker.state.right_color);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.vflip(image, self.color_picker.state.right_color);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn hflip(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.hflip(tileset, self.color_picker.state.right_color);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.hflip(image, self.color_picker.state.right_color);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn rotate(&mut self, wc: &mut WindowContext, tileset: &mut Tileset, rotation: ImageRotation) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.rotate(tileset, rotation, self.color_picker.state.right_color);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.rotate(image, rotation, self.color_picker.state.right_color);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn can_undo(&self) -> bool {
        match self.selected_tab {
            EditorTab::Tile => { self.tile_image_editor.can_undo() }
            EditorTab::GridTiles => { self.grid_image_editor.can_undo() }
            _ => { false }
        }
    }

    fn undo(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.undo(tileset);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.undo(image);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn selection_is_empty(&self) -> bool {
        match self.selected_tab {
            EditorTab::Tile => { self.tile_image_editor.selection.is_empty() }
            EditorTab::GridTiles => { self.grid_image_editor.selection.is_empty() }
            _ => { false }
        }
    }

    fn delete_selection(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.delete_selection(tileset, self.color_picker.state.right_color);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.delete_selection(image, self.color_picker.state.right_color);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn paste_pixels(&mut self, wc: &mut WindowContext, tileset: &mut Tileset, pixels: ImagePixels) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.paste_pixels(tileset, pixels);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.paste_pixels(image, pixels);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn paste(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.paste(wc, tileset);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.paste(wc, image);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn cut(&mut self, wc: &mut WindowContext, tileset: &mut Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.cut(wc, tileset, self.color_picker.state.right_color);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image_mut(tileset);
                self.grid_image_editor.cut(wc, image, self.color_picker.state.right_color);
                self.handle_grid_image_changed(wc, tileset);
            }
            _ => {}
        }
    }

    fn copy(&mut self, wc: &mut WindowContext, tileset: &Tileset) {
        match self.selected_tab {
            EditorTab::Tile => {
                self.tile_image_editor.copy(wc, tileset);
            }
            EditorTab::GridTiles => {
                let image = self.tile_grid.get_image(tileset);
                self.grid_image_editor.copy(wc, image);
            }
            _ => {}
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, tileset: &mut Tileset) {
        let import_tile_dlg_id = format!("editor_{}_import_tile", tileset.asset.id);
        if let Some(SysDialogResponse::File(filename)) = wc.sys_dialogs.get_response_for(&import_tile_dlg_id) {
            let image = match ImagePixels::load_png(&filename) {
                Ok(img) => img,
                Err(e) => {
                    wc.open_message_box("Error Loading Image", format!("Error loading {}:\n{}", filename.display(), e));
                    return;
                }
            };
            self.paste_pixels(wc, tileset, image);
        }

        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tileset", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Import...").clicked() {
                            dialogs.import_dialog.set_open(wc);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.export).max_width(14.0).max_height(14.0));
                        if ui.button("Export...").clicked() {
                            dialogs.export_dialog.set_open(wc, tileset);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.properties).max_width(14.0).max_height(14.0));
                        if ui.button("Properties...").clicked() {
                            dialogs.properties_dialog.set_open(wc, tileset, self.color_picker.state.right_color);
                        }
                    });
                });
                ui.menu_button("Edit", |ui| {
                    ui.horizontal(|ui| {
                        if ! self.can_undo() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.undo).max_width(14.0).max_height(14.0));
                        if ui.button("Undo").clicked() {
                            self.undo(wc, tileset);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if self.selection_is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.cut).max_width(14.0).max_height(14.0));
                        if ui.button("Cut").clicked() {
                            self.cut(wc, tileset);
                        }
                    });
                    ui.horizontal(|ui| {
                        if self.selection_is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.copy).max_width(14.0).max_height(14.0));
                        if ui.button("Copy").clicked() {
                            self.copy(wc, tileset);
                        }
                    });
                    ui.horizontal(|ui| {
                        if wc.image_clipboard.is_none() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.paste).max_width(14.0).max_height(14.0));
                        if ui.button("Paste").clicked() {
                            self.paste(wc, tileset);
                        }
                    });
                    ui.horizontal(|ui| {
                        if self.selection_is_empty() { ui.disable(); }
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Delete selection").clicked() {
                            self.delete_selection(wc, tileset);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.import).max_width(14.0).max_height(14.0));
                        if ui.button("Paste from file...").clicked() {
                            wc.sys_dialogs.open_file(
                                Some(wc.egui.window),
                                import_tile_dlg_id,
                                "tileset",
                                "Paste From File",
                                &[
                                    ("PNG files (*.png)", &["png"]),
                                    ("All files (*.*)", &["*"]),
                                ]
                            );
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.add).max_width(14.0).max_height(14.0));
                        if ui.button("Insert tiles...").clicked() {
                            dialogs.add_tiles_dialog.set_open(wc, AddTilesAction::Insert, self.tile_image_editor.get_selected_image(),
                                                              self.color_picker.state.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.add).max_width(14.0).max_height(14.0));
                        if ui.button("Append tiles...").clicked() {
                            dialogs.add_tiles_dialog.set_open(wc, AddTilesAction::Append, self.tile_image_editor.get_selected_image(),
                                                              self.color_picker.state.right_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(IMAGES.trash).max_width(14.0).max_height(14.0));
                        if ui.button("Remove tiles...").clicked() {
                            dialogs.rm_tiles_dialog.set_open(wc, tileset, self.tile_image_editor.get_selected_image());
                        }
                    });
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, tileset: &mut Tileset) {
        egui::Panel::top(format!("editor_panel_{}_toolbar", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                ui.label("Tool:");
                ui.add_space(1.0);
                if ui.add(egui::Button::image(IMAGES.pen)
                          .selected(self.tile_image_editor.get_tool() == ImageDrawingTool::Pencil)
                          .frame_when_inactive(self.tile_image_editor.get_tool() == ImageDrawingTool::Pencil))
                    .on_hover_text("Pencil").clicked() {
                        self.tile_image_editor.set_tool(ImageDrawingTool::Pencil);
                        self.grid_image_editor.set_tool(ImageDrawingTool::Pencil);
                    }

                if ui.add(egui::Button::image(IMAGES.fill)
                          .selected(self.tile_image_editor.get_tool() == ImageDrawingTool::Fill)
                          .frame_when_inactive(self.tile_image_editor.get_tool() == ImageDrawingTool::Fill))
                    .on_hover_text("Fill").clicked() {
                        self.tile_image_editor.set_tool(ImageDrawingTool::Fill);
                        self.grid_image_editor.set_tool(ImageDrawingTool::Fill);
                    }

                if ui.add(egui::Button::image(IMAGES.select)
                          .selected(self.tile_image_editor.get_tool() == ImageDrawingTool::Select)
                          .frame_when_inactive(self.tile_image_editor.get_tool() == ImageDrawingTool::Select))
                    .on_hover_text("Select").clicked() {
                        self.tile_image_editor.set_tool(ImageDrawingTool::Select);
                        self.grid_image_editor.set_tool(ImageDrawingTool::Select);
                    }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                if ui.add(egui::Button::image(IMAGES.v_flip)).on_hover_text("Flip vertically").clicked() {
                    self.vflip(wc, tileset);
                }
                if ui.add(egui::Button::image(IMAGES.h_flip)).on_hover_text("Flip horizontally").clicked() {
                    self.hflip(wc, tileset);
                }
                ui.add_space(5.0);
                if ui.add(egui::Button::image(IMAGES.rot_cw)).on_hover_text("Rotate 90° clockwise").clicked() {
                    self.rotate(wc, tileset, ImageRotation::CW90);
                }
                if ui.add(egui::Button::image(IMAGES.rot_ccw)).on_hover_text("Rotate 90° counter-clockwise").clicked() {
                    self.rotate(wc, tileset, ImageRotation::CCW90);
                }
                ui.spacing_mut().item_spacing = spacing;

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);
                        if ui.add(egui::Button::image(IMAGES.grid)
                                  .selected(self.tile_image_editor.display.has_bits(ImageDisplay::GRID))
                                  .frame_when_inactive(self.tile_image_editor.display.has_bits(ImageDisplay::GRID)))
                            .on_hover_text("Grid").clicked() {
                                self.tile_image_editor.toggle_display(ImageDisplay::GRID);
                                self.grid_image_editor.display = self.tile_image_editor.display;
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                                  .selected(self.tile_image_editor.display.is_transparent())
                                  .frame_when_inactive(self.tile_image_editor.display.is_transparent()))
                            .on_hover_text("Transparency").clicked() {
                                self.tile_image_editor.toggle_display(ImageDisplay::TRANSPARENT);
                                self.grid_image_editor.display = self.tile_image_editor.display;
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

    fn show_tile_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, tileset: &mut Tileset) {
        // tile picker (use the SAME ID as the other tab's panel to avoid red flashing)
        egui::Panel::left(self.tile_picker_panel_id).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.tile_picker.zoom = 4.0;
            self.tile_picker.display = self.tile_image_editor.display;
            let slot = tileset.texture_slot(self.tile_picker.display.is_transparent(), false);
            let texture = tileset.texture(wc.tex_man, wc.egui.ctx, slot);
            self.tile_picker.show(ui, wc.settings, tileset, texture, wc.settings.image_bg_color);
            if let Some(selected_image) = self.tile_picker.get_selected_image() {
                self.tile_image_editor.set_selected_image(selected_image, tileset);
            }
        });

        // tile editor
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let colors = (self.color_picker.state.left_color, self.color_picker.state.right_color);
            self.tile_image_editor.show(ui, wc, tileset, colors);
            if self.tile_image_editor.has_image_changed() {
                self.grid_image_editor.set_image_changed();
            }
            self.color_picker.maybe_set_colors(
                self.tile_image_editor.pick_left_color.take(),
                self.tile_image_editor.pick_right_color.take()
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            self.tile_image_editor.handle_keyboard(ui, wc, tileset, self.color_picker.state.right_color);
        }
    }

    fn show_grid_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, tileset: &mut Tileset) {
        // grid tile picker (use the SAME ID as the other tab's panel to avoid red flashing)
        egui::Panel::left(self.tile_picker_panel_id).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.grid_tile_picker.zoom = 4.0;
            self.grid_tile_picker.display = self.grid_image_editor.display;
            let slot = tileset.texture_slot(self.grid_tile_picker.display.is_transparent(), false);
            let texture = tileset.texture(wc.tex_man, wc.egui.ctx, slot);
            self.grid_tile_picker.show(ui, wc.settings, tileset, texture, wc.settings.image_bg_color);
            self.tile_grid_editor.left_selected_tile = self.grid_tile_picker.get_selected_image();
            self.tile_grid_editor.right_selected_tile = self.grid_tile_picker.get_selected_image_right();
        });

        // toolbar
        egui::Panel::top(format!("editor_panel_{}_grid_tab_toolbar", self.asset_id)).show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui.button("\u{2796}").clicked() && self.tile_grid.width > 1 {
                    self.tile_grid.resize(tileset, self.tile_grid.width-1, self.tile_grid.height);
                    self.grid_image_editor.set_image_changed();
                }
                ui.label(format!("{}", self.tile_grid.width));
                if ui.button("\u{2795}").clicked() && self.tile_grid.width < Self::MAX_TILE_GRID_WIDTH {
                    self.tile_grid.resize(tileset, self.tile_grid.width+1, self.tile_grid.height);
                    self.grid_image_editor.set_image_changed();
                }

                ui.separator();
                ui.label("Height:");

                if ui.button("\u{2796}").clicked() && self.tile_grid.height > 1 {
                    self.tile_grid.resize(tileset, self.tile_grid.width, self.tile_grid.height-1);
                    self.grid_image_editor.set_image_changed();
                }
                ui.label(format!("{}", self.tile_grid.height));
                if ui.button("\u{2795}").clicked() && self.tile_grid.height < Self::MAX_TILE_GRID_HEIGHT {
                    self.tile_grid.resize(tileset, self.tile_grid.width, self.tile_grid.height+1);
                    self.grid_image_editor.set_image_changed();
                }
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            match self.tile_grid_editor.show(ui, wc, &mut self.tile_grid, tileset) {
                TileGridEditorAction::None => {}
                TileGridEditorAction::PickLeftTile(tile) => {
                    self.grid_tile_picker.set_selected_image(tile);
                }
                TileGridEditorAction::PickRightTile(tile) => {
                    self.grid_tile_picker.set_selected_image_right(tile);
                }
                TileGridEditorAction::SetTile => {
                    self.tile_grid.tileset_to_image(tileset);
                    self.grid_image_editor.set_image_changed();
                }
            }
        });
    }

    fn show_grid_tiles_tab(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, tileset: &mut Tileset) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let colors = (self.color_picker.state.left_color, self.color_picker.state.right_color);

            let grid_image = self.tile_grid.get_image_mut(tileset);
            self.grid_image_editor.show(ui, wc, grid_image, colors);
            if self.grid_image_editor.has_image_changed() {
                self.handle_grid_image_changed(wc, tileset);
            }
            self.color_picker.maybe_set_colors(
                self.grid_image_editor.pick_left_color.take(),
                self.grid_image_editor.pick_right_color.take(),
            );
        });

        // keyboard shortcuts
        if wc.is_editor_on_top(self.asset_id) {
            let image = self.tile_grid.get_image_mut(tileset);
            if ! matches!(self.grid_image_editor.handle_keyboard(ui, wc, image, self.color_picker.state.right_color),
                          ImageEditorAction::None) {
                self.handle_grid_image_changed(wc, tileset);
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, tileset: &mut Tileset) {
        self.show_menu_bar(ui, wc, dialogs, tileset);
        self.show_toolbar(ui, wc, tileset);

        // color picker:
        egui::Panel::right(format!("editor_panel_{}_right", self.asset_id)).resizable(false).show_inside(ui, |ui| {
            ui.add_space(5.0);
            match self.color_picker.show(ui, wc) {
                ColorPickerResponse::None => {}
                ColorPickerResponse::CreateColorset => {
                    dialogs.create_colorset_dialog.set_open(wc, self.tile_image_editor.get_selected_image());
                }
            }
        });

        // tabs
        egui::Panel::top(format!("editor_panel_{}_tabs", self.asset_id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::Tile), "Tile").clicked() {
                    self.selected_tab = EditorTab::Tile;
                }
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::Grid), "Grid").clicked() {
                    self.selected_tab = EditorTab::Grid;
                }
                if ui.selectable_label(matches!(self.selected_tab, EditorTab::GridTiles), "Grid Tiles").clicked() {
                    self.selected_tab = EditorTab::GridTiles;
                }
            });
        });
        match self.selected_tab {
            EditorTab::Tile => { self.show_tile_tab(ui, wc, tileset); }
            EditorTab::Grid => { self.show_grid_tab(ui, wc, tileset); }
            EditorTab::GridTiles => { self.show_grid_tiles_tab(ui, wc, tileset); }
        }
    }
}
