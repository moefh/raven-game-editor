use std::collections::HashMap;

use crate::data_asset::{DataAssetId, DataAssetStore};
use crate::editors::{
    DataAssetEditor, TilesetEditor, MapDataEditor, RoomEditor,
    SpriteEditor, SpriteAnimationEditor, SfxEditor, ModDataEditor,
    FontEditor, PropFontEditor,
};

pub struct AssetEditors {
    pub egui_id_to_asset_id: HashMap<egui::Id, DataAssetId>,
    pub tilesets: HashMap<DataAssetId, TilesetEditor>,
    pub maps: HashMap<DataAssetId, MapDataEditor>,
    pub rooms: HashMap<DataAssetId, RoomEditor>,
    pub sprites: HashMap<DataAssetId, SpriteEditor>,
    pub animations: HashMap<DataAssetId, SpriteAnimationEditor>,
    pub sfxs: HashMap<DataAssetId, SfxEditor>,
    pub mods: HashMap<DataAssetId, ModDataEditor>,
    pub fonts: HashMap<DataAssetId, FontEditor>,
    pub prop_fonts: HashMap<DataAssetId, PropFontEditor>,
}

impl AssetEditors {

    pub fn new() -> Self {
        AssetEditors {
            egui_id_to_asset_id: HashMap::new(),
            tilesets: HashMap::new(),
            maps: HashMap::new(),
            rooms: HashMap::new(),
            sprites: HashMap::new(),
            animations: HashMap::new(),
            sfxs: HashMap::new(),
            mods: HashMap::new(),
            fonts: HashMap::new(),
            prop_fonts: HashMap::new(),
        }
    }

    pub fn get_top_editor_asset_id(&self, ctx: &egui::Context) -> Option<DataAssetId> {
        ctx.memory(|mem| {
            mem.layer_ids().fold(None, |top, layer_id| self.egui_id_to_asset_id.get(&layer_id.id).copied().or(top))
        })
    }

    pub fn clear(&mut self) {
        self.egui_id_to_asset_id.clear();
        self.tilesets.clear();
        self.maps.clear();
        self.rooms.clear();
        self.sprites.clear();
        self.animations.clear();
        self.sfxs.clear();
        self.mods.clear();
        self.fonts.clear();
        self.prop_fonts.clear();
    }

    pub fn create_editors_for_new_store(&mut self, store: &DataAssetStore) {
        for id in store.asset_ids.tilesets.iter().copied() { self.add_tileset(id); }
        for id in store.asset_ids.maps.iter().copied() { self.add_map(id); }
        for id in store.asset_ids.rooms.iter().copied() { self.add_room(id); }
        for id in store.asset_ids.sprites.iter().copied() { self.add_sprite(id); }
        for id in store.asset_ids.animations.iter().copied() { self.add_animation(id); }
        for id in store.asset_ids.sfxs.iter().copied() { self.add_sfx(id); }
        for id in store.asset_ids.mods.iter().copied() { self.add_mod(id); }
        for id in store.asset_ids.fonts.iter().copied() { self.add_font(id); }
        for id in store.asset_ids.prop_fonts.iter().copied() { self.add_prop_font(id); }
    }

    pub fn get_editor(&self, id: DataAssetId) -> Option<&DataAssetEditor> {
        if let Some(editor) = self.tilesets.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.maps.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.rooms.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.sprites.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.animations.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.sfxs.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.mods.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.fonts.get(&id) { return Some(&editor.asset); }
        if let Some(editor) = self.prop_fonts.get(&id) { return Some(&editor.asset); }
        None
    }

    pub fn get_editor_mut(&mut self, id: DataAssetId) -> Option<&mut DataAssetEditor> {
        if let Some(editor) = self.tilesets.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.maps.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.rooms.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.sprites.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.animations.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.sfxs.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.mods.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.fonts.get_mut(&id) { return Some(&mut editor.asset); }
        if let Some(editor) = self.prop_fonts.get_mut(&id) { return Some(&mut editor.asset); }
        None
    }

    pub fn remove_editor(&mut self, id: DataAssetId) -> bool {
        if let Some(editor) = self.tilesets.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.maps.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.rooms.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.sprites.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.animations.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.sfxs.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.mods.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        if let Some(editor) = self.prop_fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.asset.egui_id); return true; }
        false
    }

    pub fn add_tileset(&mut self, id: DataAssetId) {
        let editor = TilesetEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.tilesets.insert(id, editor);
    }

    pub fn add_map(&mut self, id: DataAssetId) {
        let editor = MapDataEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.maps.insert(id, editor);
    }

    pub fn add_room(&mut self, id: DataAssetId) {
        let editor = RoomEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.rooms.insert(id, editor);
    }

    pub fn add_sprite(&mut self, id: DataAssetId) {
        let editor = SpriteEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.sprites.insert(id, editor);
    }

    pub fn add_animation(&mut self, id: DataAssetId) {
        let editor = SpriteAnimationEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.animations.insert(id, editor);
    }

    pub fn add_sfx(&mut self, id: DataAssetId) {
        let editor = SfxEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.sfxs.insert(id, editor);
    }

    pub fn add_mod(&mut self, id: DataAssetId) {
        let editor = ModDataEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.mods.insert(id, editor);
    }

    pub fn add_font(&mut self, id: DataAssetId) {
        let editor = FontEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.fonts.insert(id, editor);
    }

    pub fn add_prop_font(&mut self, id: DataAssetId) {
        let editor = PropFontEditor::new(id, false);
        self.egui_id_to_asset_id.insert(editor.asset.egui_id, editor.asset.id);
        self.prop_fonts.insert(id, editor);
    }
}
