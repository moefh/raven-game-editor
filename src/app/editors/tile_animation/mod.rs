mod properties;

use crate::misc::IMAGES;
use crate::image::{
    ImageCollection,
};
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
    AssetEditorBase,
    WindowContext,
};
use super::widgets::{
    ImagePickerWidget,
    ImageEditorWidget,
    SpriteFrameListView,
};
use super::super::menu_item;

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

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, tanim: &TileAnimation, base: &AssetEditorBase) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, tanim.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", tanim.asset.id)).frame(bottom_frame).show(ui, |ui| {
            let dirty = if base.is_dirty() { " (modified)" } else { "" };
            ui.add(egui::Label::new(format!(
                "{} bytes {}",
                tanim.data_size(),
                dirty
            )).truncate());
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
            Self::show_footer(ui, wc, tanim, base);
            self.editor.show(ui, wc, &mut self.dialogs, tanim, tilesets);
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
    parent_tile_picker_panel_id: egui::Id,
    anim_tile_picker_panel_id: egui::Id,
    anim_loop_len_panel_id: egui::Id,
    parent_tile_picker: ImagePickerWidget,
    anim_tile_view: SpriteFrameListView,
    frame_indices: Vec<SpriteAnimationFrame>,
    image_editor: ImageEditorWidget<Tileset>,
}

impl Editor {
    pub fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            header_panel_id: egui::Id::new(format!("editor_panel_{}_header", asset_id)),
            parent_tile_picker_panel_id: egui::Id::new(format!("editor_panel_{}_parent_tile_picker", asset_id)),
            anim_tile_picker_panel_id: egui::Id::new(format!("editor_panel_{}_anim_tile_picker", asset_id)),
            anim_loop_len_panel_id: egui::Id::new(format!("editor_panel_{}_anim_loop_len", asset_id)),
            parent_tile_picker: ImagePickerWidget::new(),
            anim_tile_view: SpriteFrameListView::new(true, false),
            frame_indices: Vec::with_capacity(255),
            image_editor: ImageEditorWidget::new(),
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

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        tanim: &mut TileAnimation,
        tilesets: &mut AssetList<Tileset>
    ) {
        self.fix_frame_indices(tanim.anim_tileset_id, tilesets);

        // menu bar
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

        // parent tiles
        egui::Panel::left(self.parent_tile_picker_panel_id).resizable(false).show(ui, |ui| {
            ui.add_space(5.0);
            if let Some(tileset) = tilesets.get(&tanim.parent_tileset_id) {
                self.parent_tile_picker.zoom = 4.0;
                self.parent_tile_picker.display = self.image_editor.display;
                let slot = tileset.texture_slot(self.parent_tile_picker.display.is_transparent(), false);
                let texture = tileset.texture(wc.tex_man, wc.egui.ctx, slot);
                self.parent_tile_picker.show(ui, wc.settings, tileset, texture, wc.settings.image_bg_color);
                if let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
                    let Some(tloop) = tanim.loops.get(selected_tile as usize) &&
                    self.anim_tile_view.selected_frame != tloop.start as usize {
                        self.anim_tile_view.selected_frame = tloop.start as usize;
                        self.anim_tile_view.scroll_to_selection();
                    }
            }
        });

        // selected loop
        if let Some(selected_tile) = self.parent_tile_picker.get_selected_image_l() &&
            let Some(tloop) = tanim.loops.get_mut(selected_tile as usize) &&
            let Some(anim_tileset) = tilesets.get_mut(&tanim.anim_tileset_id) {
                let (mut loop_start, mut loop_len) = (tloop.start as u32, tloop.len as u32);
                let max_loop_len = anim_tileset.num_tiles.saturating_sub(loop_start);
                egui::Panel::top(self.anim_tile_picker_panel_id).show(ui, |ui| {
                    ui.add_space(5.0);
                    ui.label("Loop start:");
                    self.anim_tile_view.show(
                        ui,
                        wc,
                        &self.frame_indices,
                        0,
                        anim_tileset,
                        true  //self.image_editor.display.is_transparent(),
                    );
                    loop_start = self.anim_tile_view.selected_frame as u32;
                    if loop_start + loop_len > anim_tileset.num_tiles {
                        loop_len = anim_tileset.num_tiles.saturating_sub(loop_start);
                    }
                });
                egui::Panel::top(self.anim_loop_len_panel_id).show(ui, |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("Loop Length:");
                        if ui.button("\u{2796}").clicked() && loop_len > 0 {
                            loop_len -= 1;
                        }
                        ui.label(format!("{}", loop_len));
                        if ui.button("\u{2795}").clicked() && loop_len < max_loop_len {
                            loop_len += 1;
                        }
                    });
                    ui.add_space(5.0);
                });
                tloop.start = (loop_start & 0xff) as u8;
                tloop.len = (loop_len & 0xff) as u8;

                egui::CentralPanel::default().show(ui, |ui| {
                    self.image_editor.set_selected_image(loop_start, anim_tileset);
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
