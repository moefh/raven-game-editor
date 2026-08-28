mod properties;

use crate::misc::IMAGES;
use crate::data_asset::{
    DataAssetId,
    GenericAsset,
    AssetList,
    AssetIdList,
    AssetIdCollection,
    TileAnimation,
    Tileset,
    SpriteAnimationFrame,
};

use super::{
    IMAGE_ZOOM_OPTIONS,
    get_animation_step,
    ImageZoomOption,
    AssetEditorBase,
    WindowContext,
    TilesetTileFixer,
};
use super::widgets::{
    ImagePickerWidget,
    ImageEditorWidget,
    SpriteFrameListView,
    TilePickerPopupWidget,
    ImageDisplay,
};
use super::super::{
    menu_item,
};

use properties::PropertiesDialog;

pub struct TileAnimationEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl TileAnimationEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        TileAnimationEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(),
        }
    }

    pub fn prepare_for_saving(&mut self, _tanim: &mut TileAnimation) {
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, editor: &Editor, tanim: &TileAnimation, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, tanim.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", tanim.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(format!(
                    "{} bytes{}",
                    tanim.data_size(),
                    dirty
                )).truncate());
                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("[tile {}]", editor.parent_tile_picker.get_selected_image_l().unwrap_or(0)));
                    });
                });
            });
        });
    }

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        tanim: &mut TileAnimation,
        asset_ids: &AssetIdCollection,
        tilesets: &mut AssetList<Tileset>
    ) {
        self.dialogs.show(wc, &mut self.editor, tanim, &asset_ids.tilesets, tilesets);

        self.base.show_window(wc, tanim, [500.0, 400.0], [500.0, 400.0], |ui, wc, tanim, base| {
            Self::show_footer(ui, wc, &self.editor, tanim, base);
            self.editor.show(ui, wc, &mut self.dialogs, tanim, tilesets);
        });
    }
}

impl TilesetTileFixer for TileAnimationEditor {
    fn move_tile(&mut self, _tileset_id: DataAssetId, _old_index: u8, _new_index: u8) {
        self.editor.reload_edit_loop = true;
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

    pub fn show(
        &mut self,
        wc: &mut WindowContext,
        _editor: &mut Editor,
        tanim: &mut TileAnimation,
        tileset_ids: &AssetIdList,
        tilesets: &AssetList<Tileset>
    ) {
        if let Some(dlg) = &mut self.properties_dialog && dlg.open {
            dlg.show(wc, tanim, tileset_ids, tilesets);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    header_panel_id: egui::Id,
    playback_toolbar_panel_id: egui::Id,
    display_toolbar_panel_id: egui::Id,
    parent_tile_picker_panel_id: egui::Id,
    anim_tile_picker_panel_id: egui::Id,
    anim_loop_len_panel_id: egui::Id,
    parent_tile_picker: ImagePickerWidget,
    parent_tile_picker_popup: TilePickerPopupWidget,
    anim_tile_view: SpriteFrameListView,
    frame_indices: Vec<SpriteAnimationFrame>,
    image_editor: ImageEditorWidget<Tileset>,
    edit_loop_len: u32,
    reload_edit_loop: bool,
    playing: bool,
    reverse_play: bool,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            header_panel_id: egui::Id::new(format!("editor_panel_{}_header", asset_id)),
            playback_toolbar_panel_id: egui::Id::new(format!("editor_panel_{}_playback_toolbar", asset_id)),
            display_toolbar_panel_id: egui::Id::new(format!("editor_panel_{}_display_toolbar", asset_id)),
            parent_tile_picker_panel_id: egui::Id::new(format!("editor_panel_{}_parent_tile_picker", asset_id)),
            parent_tile_picker_popup: TilePickerPopupWidget::new(egui::Id::new(format!("editor_panel_{}_parent_tile_picker", asset_id))),
            anim_tile_picker_panel_id: egui::Id::new(format!("editor_panel_{}_anim_tile_picker", asset_id)),
            anim_loop_len_panel_id: egui::Id::new(format!("editor_panel_{}_anim_loop_len", asset_id)),
            parent_tile_picker: ImagePickerWidget::new(),
            anim_tile_view: SpriteFrameListView::new(true, false),
            frame_indices: Vec::with_capacity(255),
            image_editor: ImageEditorWidget::new().readonly().with_image_display(ImageDisplay::new(ImageDisplay::TRANSPARENT)),
            edit_loop_len: 0,
            reload_edit_loop: true,
            playing: false,
            reverse_play: false,
        }
    }

    fn fix_frame_indices(&mut self, tileset_id: DataAssetId, tilesets: &AssetList<Tileset>) {
        if let Some(tileset) = tilesets.get(&tileset_id) && self.frame_indices.len() != tileset.num_tiles as usize {
            self.frame_indices.clear();
            self.frame_indices.extend((0..tileset.num_tiles).map(|n| {
                SpriteAnimationFrame { head_index: Some((n & 0xff) as u8), foot_index: None }
            }));
        }
    }

    fn show_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        tanim: &mut TileAnimation
    ) {
        egui::Panel::top(self.header_panel_id).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tile Animation", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        let dlg = dialogs.properties_dialog.get_or_insert_with(|| {
                            PropertiesDialog::new(tanim.asset.id, tanim.parent_tileset_id, tanim.anim_tileset_id)
                        });
                        dlg.set_open(wc, tanim);
                    }
                });
            });
        });
    }

    fn show_playback_toolbar(&mut self, ui: &mut egui::Ui, _wc: &WindowContext, tanim: &TileAnimation) {
        egui::Panel::top(self.playback_toolbar_panel_id).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let can_play = self.parent_tile_picker.get_selected_image_l().and_then(|sel_tile| {
                    tanim.loops.get(sel_tile as usize)
                }).is_some_and(|tloop| tloop.len != 0);
                if ui.add_enabled(
                    can_play && (! self.playing || ! self.reverse_play),
                    egui::Button::new("\u{23f4}")
                ).on_hover_text("Play reversed").clicked() {
                    self.playing = true;
                    self.reverse_play = true;
                }
                if ui.add_enabled(
                    self.playing,
                    egui::Button::new("\u{23f8}")
                ).on_hover_text("Pause").clicked() {
                    self.playing = false;
                }
                if ui.add_enabled(
                    can_play && (! self.playing || self.reverse_play),
                    egui::Button::new("\u{23f5}")
                ).on_hover_text("Play").clicked() {
                    self.playing = true;
                    self.reverse_play = false;
                }
            });
            ui.add_space(0.0); // don't remove this
        });
    }

    fn show_display_toolbar(&mut self, ui: &mut egui::Ui, _wc: &WindowContext, tanim: &TileAnimation) {
        egui::Panel::top(self.display_toolbar_panel_id).show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
                    let Some(tloop) = tanim.loops.get(selected_tile as usize) &&
                    tloop.len != 0 {
                        ui.label(format!("Animation: {} frames", tloop.len));
                    } else {
                        ui.label("No animation");
                    }

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(2.0);
                        let spacing = ui.spacing().item_spacing;
                        ui.spacing_mut().item_spacing = egui::Vec2::new(1.0, 0.0);

                        if ui.add(egui::Button::image(IMAGES.grid)
                            .selected(self.image_editor.display.has_bits(ImageDisplay::GRID))
                            .frame_when_inactive(self.image_editor.display.has_bits(ImageDisplay::GRID)))
                            .on_hover_text("Grid").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::GRID);
                            }
                        if ui.add(egui::Button::image(IMAGES.transparency)
                            .selected(self.image_editor.display.is_transparent())
                            .frame_when_inactive(self.image_editor.display.is_transparent()))
                            .on_hover_text("Transparency").clicked() {
                                self.image_editor.toggle_display(ImageDisplay::TRANSPARENT);
                            }
                        ui.label("Display:");

                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let mut cur_zoom_option = ImageZoomOption::from_image_editor_zoom(self.image_editor.zoom);
                        egui::ComboBox::from_id_salt(format!("pal_sprite_editor_{}_zoom_combo", self.asset_id))
                            .selected_text(cur_zoom_option.name())
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for option in IMAGE_ZOOM_OPTIONS {
                                    if option.is_custom() && ! cur_zoom_option.is_custom() { continue; }
                                    ui.selectable_value(&mut cur_zoom_option, option, option.name());
                                }
                            });
                        let new_zoom = cur_zoom_option.image_editor_zoom(self.image_editor.zoom);
                        self.image_editor.zoom = new_zoom;
                        ui.add_space(1.0);
                        ui.label("Zoom:");

                        ui.spacing_mut().item_spacing = spacing;
                    });
                });
            });
            ui.add_space(0.0);  // don't remove this, it's necessary
        });
    }

    fn show_parent_tile_picker(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        tanim: &mut TileAnimation,
        tilesets: &mut AssetList<Tileset>
    ) {
        egui::Panel::left(self.parent_tile_picker_panel_id).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            if let Some(anim_tileset) = tilesets.get(&tanim.parent_tileset_id) {
                if let Some(tile) = self.parent_tile_picker_popup.show_anchor(
                    ui,
                    wc,
                    "Select...",
                    anim_tileset,
                    self.parent_tile_picker.get_selected_image_l()
                ) {
                    self.parent_tile_picker.set_selected_image_l(Some(tile));
                    self.reload_edit_loop = true;
                }
                self.parent_tile_picker.display = self.image_editor.display;
                self.parent_tile_picker.show(ui, wc, anim_tileset);
                if self.parent_tile_picker.image_l_picked {
                    self.reload_edit_loop = true;
                }
            }
        });
    }

    fn ensure_valid_selected_tiles(&mut self, tanim: &mut TileAnimation, tilesets: &mut AssetList<Tileset>) {
        if let Some(anim_tileset) = tilesets.get(&tanim.anim_tileset_id) &&
            self.image_editor.get_selected_image() >= anim_tileset.num_tiles {
                self.image_editor.set_selected_image(anim_tileset.num_tiles.saturating_sub(1), anim_tileset);
            }
        if let Some(parent_tileset) = tilesets.get(&tanim.parent_tileset_id) &&
            let Some(tile) = self.parent_tile_picker.get_selected_image_l() &&
            tile >= parent_tileset.num_tiles as u32 {
                self.parent_tile_picker.set_selected_image_l(Some((parent_tileset.num_tiles as u32).saturating_sub(1)));
            }
    }


    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        tanim: &mut TileAnimation,
        tilesets: &mut AssetList<Tileset>
    ) {
        self.fix_frame_indices(tanim.anim_tileset_id, tilesets);
        self.ensure_valid_selected_tiles(tanim, tilesets);
        if self.reload_edit_loop &&
            let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
            let Some(tloop) = tanim.loops.get_mut(selected_tile as usize) {
                self.reload_edit_loop = false;
                self.edit_loop_len = tloop.len as u32;
                if self.edit_loop_len != 0 {
                    if let Some(anim_tileset) = tilesets.get(&tanim.anim_tileset_id) {
                        self.image_editor.set_selected_image(tloop.start as u32, anim_tileset);
                    }
                    if self.anim_tile_view.selected_frame != tloop.start as usize {
                        self.anim_tile_view.selected_frame = tloop.start as usize;
                        self.anim_tile_view.scroll_to_selection();
                    }
                }
            }

        self.show_menu_bar(ui, wc, dialogs, tanim);
        self.show_parent_tile_picker(ui, wc, tanim, tilesets);
        self.show_playback_toolbar(ui, wc, tanim);
        self.show_display_toolbar(ui, wc, tanim);

        // selected loop
        let loop_start = self.anim_tile_view.selected_frame as u32;
        let max_loop_len = if let Some(anim_tileset) = tilesets.get(&tanim.anim_tileset_id) {
            if loop_start + self.edit_loop_len > anim_tileset.num_tiles {
                self.edit_loop_len = anim_tileset.num_tiles.saturating_sub(loop_start);
            }
            anim_tileset.num_tiles.saturating_sub(loop_start)
        } else {
            0
        };
        egui::Panel::bottom(self.anim_loop_len_panel_id).show(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Loop Length:");
                if ui.button("\u{2796}").clicked() && self.edit_loop_len > 0 {
                    self.edit_loop_len -= 1;
                }
                ui.label(format!("{}", self.edit_loop_len));
                if ui.button("\u{2795}").clicked() && self.edit_loop_len < max_loop_len {
                    self.edit_loop_len += 1;
                }

                ui.add_space(20.0);

                if let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
                    let Some(tloop) = tanim.loops.get_mut(selected_tile as usize) {
                        let start_changed = loop_start != tloop.start as u32;
                        let len_changed = self.edit_loop_len != tloop.len as u32;
                        if ui.add_enabled(
                            len_changed || (start_changed && (self.edit_loop_len != 0 || tloop.len != 0)),
                            egui::Button::new("Set Loop")
                        ).clicked() {
                            tloop.start = if self.edit_loop_len == 0 { 0 } else { (loop_start & 0xff) as u8 };
                            tloop.len = (self.edit_loop_len & 0xff) as u8;
                        }
                        if ui.add_enabled(
                            tloop.len != 0,
                            egui::Button::new("Remove Loop")
                        ).clicked() {
                            tloop.len = 0;
                        }
                    }
            });
            ui.add_space(5.0);
        });

        if let Some(anim_tileset) = tilesets.get_mut(&tanim.anim_tileset_id) {
            egui::Panel::bottom(self.anim_tile_picker_panel_id).show(ui, |ui| {
                ui.add_space(5.0);
                ui.label("Loop start:");
                self.anim_tile_view.show(
                    ui,
                    wc,
                    &self.frame_indices,
                    0,
                    anim_tileset,
                    self.image_editor.display.is_transparent(),
                );
            });
            egui::CentralPanel::default().show(ui, |ui| {
                if self.playing &&
                    let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
                    let Some(tloop) = tanim.loops.get_mut(selected_tile as usize) &&
                    tloop.len != 0 {
                        let animation_step = get_animation_step(wc);
                        let loop_len = tloop.len as u32;
                        let play_tile = if self.reverse_play {
                            tloop.start.saturating_add((loop_len - 1 - (animation_step + 1) % loop_len) as u8) as u32
                        } else {
                            tloop.start.saturating_add((animation_step % loop_len as u32) as u8) as u32
                        };
                        self.image_editor.set_selected_image(play_tile, anim_tileset);
                        wc.request_map_animation_repaint();
                    } else {
                        self.image_editor.set_selected_image(loop_start, anim_tileset);
                    }
                let colors = (0xff, 0xff);
                self.image_editor.show(ui, wc, anim_tileset, colors);
            });
        }

        // handle keyboard
        if wc.is_editor_on_top(self.asset_id) && let Some(anim_tileset) = tilesets.get_mut(&tanim.anim_tileset_id) {
            self.image_editor.handle_keyboard(ui, wc, anim_tileset, 0xff);
        }
    }
}
