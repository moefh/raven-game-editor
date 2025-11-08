use std::collections::HashMap;

use crate::data_asset::{DataAssetId, DataAssetStore};
use crate::editors::{
    DataAssetEditor, TilesetEditor, MapDataEditor, RoomEditor,
    SpriteEditor, SpriteAnimationEditor, SfxEditor, ModDataEditor,
    FontEditor, PropFontEditor,
};

pub struct AssetEditors {
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

    pub fn clear(&mut self) {
        self.tilesets = HashMap::new();
        self.maps = HashMap::new();
        self.rooms = HashMap::new();
        self.sprites = HashMap::new();
        self.animations = HashMap::new();
        self.sfxs = HashMap::new();
        self.mods = HashMap::new();
        self.fonts = HashMap::new();
        self.prop_fonts = HashMap::new();
    }

    pub fn create_editors_for_new_store(&mut self, store: &DataAssetStore) {
        for &id in store.asset_ids.tilesets.iter() { self.add_tileset(id); }
        for &id in store.asset_ids.maps.iter() { self.add_map(id); }
        for &id in store.asset_ids.rooms.iter() { self.add_room(id); }
        for &id in store.asset_ids.sprites.iter() { self.add_sprite(id); }
        for &id in store.asset_ids.animations.iter() { self.add_animation(id); }
        for &id in store.asset_ids.sfxs.iter() { self.add_sfx(id); }
        for &id in store.asset_ids.mods.iter() { self.add_mod(id); }
        for &id in store.asset_ids.fonts.iter() { self.add_font(id); }
        for &id in store.asset_ids.prop_fonts.iter() { self.add_prop_font(id); }
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
        if self.tilesets.remove(&id).is_some() { return true; }
        if self.maps.remove(&id).is_some() { return true; }
        if self.rooms.remove(&id).is_some() { return true; }
        if self.sprites.remove(&id).is_some() { return true; }
        if self.animations.remove(&id).is_some() { return true; }
        if self.sfxs.remove(&id).is_some() { return true; }
        if self.mods.remove(&id).is_some() { return true; }
        if self.fonts.remove(&id).is_some() { return true; }
        if self.prop_fonts.remove(&id).is_some() { return true; }
        false
    }

    pub fn add_tileset(&mut self, id: DataAssetId) {
        self.tilesets.insert(id, TilesetEditor::new(id, false));
    }

    pub fn add_map(&mut self, id: DataAssetId) {
        self.maps.insert(id, MapDataEditor::new(id, false));
    }

    pub fn add_room(&mut self, id: DataAssetId) {
        self.rooms.insert(id, RoomEditor::new(id, false));
    }

    pub fn add_sprite(&mut self, id: DataAssetId) {
        self.sprites.insert(id, SpriteEditor::new(id, false));
    }

    pub fn add_animation(&mut self, id: DataAssetId) {
        self.animations.insert(id, SpriteAnimationEditor::new(id, false));
    }

    pub fn add_sfx(&mut self, id: DataAssetId) {
        self.sfxs.insert(id, SfxEditor::new(id, false));
    }

    pub fn add_mod(&mut self, id: DataAssetId) {
        self.mods.insert(id, ModDataEditor::new(id, false));
    }

    pub fn add_font(&mut self, id: DataAssetId) {
        self.fonts.insert(id, FontEditor::new(id, false));
    }

    pub fn add_prop_font(&mut self, id: DataAssetId) {
        self.prop_fonts.insert(id, PropFontEditor::new(id, false));
    }
}
