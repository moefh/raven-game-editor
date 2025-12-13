use std::collections::HashMap;

use crate::data_asset::{DataAssetId, DataAssetStore};
use crate::editors::{
    AssetEditorBase, TilesetEditor, MapDataEditor, RoomEditor,
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
        self.clear_dirty(store);
    }

    pub fn get_editor(&self, id: DataAssetId) -> Option<&AssetEditorBase> {
        if let Some(editor) = self.tilesets.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.maps.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.rooms.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.sprites.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.animations.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.sfxs.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.mods.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.fonts.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.prop_fonts.get(&id) { return Some(&editor.base); }
        None
    }

    pub fn get_editor_mut(&mut self, id: DataAssetId) -> Option<&mut AssetEditorBase> {
        if let Some(editor) = self.tilesets.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.maps.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.rooms.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.sprites.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.animations.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.sfxs.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.mods.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.fonts.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.prop_fonts.get_mut(&id) { return Some(&mut editor.base); }
        None
    }

    pub fn remove_editor(&mut self, id: DataAssetId) -> bool {
        if let Some(editor) = self.tilesets.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.maps.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.rooms.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.sprites.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.animations.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.sfxs.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.mods.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.prop_fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        false
    }

    pub fn add_tileset(&mut self, id: DataAssetId) -> egui::Id {
        let editor = TilesetEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.tilesets.insert(id, editor);
        egui_id
    }

    pub fn add_map(&mut self, id: DataAssetId) -> egui::Id {
        let editor = MapDataEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.maps.insert(id, editor);
        egui_id
    }

    pub fn add_room(&mut self, id: DataAssetId) -> egui::Id {
        let editor = RoomEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.rooms.insert(id, editor);
        egui_id
    }

    pub fn add_sprite(&mut self, id: DataAssetId) -> egui::Id {
        let editor = SpriteEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.sprites.insert(id, editor);
        egui_id
    }

    pub fn add_animation(&mut self, id: DataAssetId) -> egui::Id {
        let editor = SpriteAnimationEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.animations.insert(id, editor);
        egui_id
    }

    pub fn add_sfx(&mut self, id: DataAssetId) -> egui::Id {
        let editor = SfxEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.sfxs.insert(id, editor);
        egui_id
    }

    pub fn add_mod(&mut self, id: DataAssetId) -> egui::Id {
        let editor = ModDataEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.mods.insert(id, editor);
        egui_id
    }

    pub fn add_font(&mut self, id: DataAssetId) -> egui::Id {
        let editor = FontEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.fonts.insert(id, editor);
        egui_id
    }

    pub fn add_prop_font(&mut self, id: DataAssetId) -> egui::Id {
        let editor = PropFontEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.prop_fonts.insert(id, editor);
        egui_id
    }

    pub fn update_dirty(&mut self, store: &DataAssetStore) {
        for tileset in store.assets.tilesets.iter() {
            if let Some(editor) = self.tilesets.get_mut(&tileset.asset.id) { editor.base.update_dirty(tileset); }
        }
        for map in store.assets.maps.iter() {
            if let Some(editor) = self.maps.get_mut(&map.asset.id) { editor.base.update_dirty(map); }
        }
        for room in store.assets.rooms.iter() {
            if let Some(editor) = self.rooms.get_mut(&room.asset.id) { editor.base.update_dirty(room); }
        }
        for sprite in store.assets.sprites.iter() {
            if let Some(editor) = self.sprites.get_mut(&sprite.asset.id) { editor.base.update_dirty(sprite); }
        }
        for anim in store.assets.animations.iter() {
            if let Some(editor) = self.animations.get_mut(&anim.asset.id) { editor.base.update_dirty(anim); }
        }
        for sfx in store.assets.sfxs.iter() {
            if let Some(editor) = self.sfxs.get_mut(&sfx.asset.id) { editor.base.update_dirty(sfx); }
        }
        for mod_data in store.assets.mods.iter() {
            if let Some(editor) = self.mods.get_mut(&mod_data.asset.id) { editor.base.update_dirty(mod_data); }
        }
        for font in store.assets.fonts.iter() {
            if let Some(editor) = self.fonts.get_mut(&font.asset.id) { editor.base.update_dirty(font); }
        }
        for pfont in store.assets.prop_fonts.iter() {
            if let Some(editor) = self.prop_fonts.get_mut(&pfont.asset.id) { editor.base.update_dirty(pfont); }
        }
    }

    pub fn clear_dirty(&mut self, store: &DataAssetStore) {
        for tileset in store.assets.tilesets.iter() {
            if let Some(editor) = self.tilesets.get_mut(&tileset.asset.id) { editor.base.clear_dirty(tileset); }
        }
        for map in store.assets.maps.iter() {
            if let Some(editor) = self.maps.get_mut(&map.asset.id) { editor.base.clear_dirty(map); }
        }
        for room in store.assets.rooms.iter() {
            if let Some(editor) = self.rooms.get_mut(&room.asset.id) { editor.base.clear_dirty(room); }
        }
        for sprite in store.assets.sprites.iter() {
            if let Some(editor) = self.sprites.get_mut(&sprite.asset.id) { editor.base.clear_dirty(sprite); }
        }
        for anim in store.assets.animations.iter() {
            if let Some(editor) = self.animations.get_mut(&anim.asset.id) { editor.base.clear_dirty(anim); }
        }
        for sfx in store.assets.sfxs.iter() {
            if let Some(editor) = self.sfxs.get_mut(&sfx.asset.id) { editor.base.clear_dirty(sfx); }
        }
        for mod_data in store.assets.mods.iter() {
            if let Some(editor) = self.mods.get_mut(&mod_data.asset.id) { editor.base.clear_dirty(mod_data); }
        }
        for font in store.assets.fonts.iter() {
            if let Some(editor) = self.fonts.get_mut(&font.asset.id) { editor.base.clear_dirty(font); }
        }
        for pfont in store.assets.prop_fonts.iter() {
            if let Some(editor) = self.prop_fonts.get_mut(&pfont.asset.id) { editor.base.clear_dirty(pfont); }
        }
    }

    pub fn is_dirty(&mut self) -> bool {
        if self.tilesets.values().any(|e| e.base.is_dirty()) { return true; }
        if self.maps.values().any(|e| e.base.is_dirty()) { return true; }
        if self.rooms.values().any(|e| e.base.is_dirty()) { return true; }
        if self.sprites.values().any(|e| e.base.is_dirty()) { return true; }
        if self.animations.values().any(|e| e.base.is_dirty()) { return true; }
        if self.sfxs.values().any(|e| e.base.is_dirty()) { return true; }
        if self.mods.values().any(|e| e.base.is_dirty()) { return true; }
        if self.fonts.values().any(|e| e.base.is_dirty()) { return true; }
        if self.prop_fonts.values().any(|e| e.base.is_dirty()) { return true; }
        false
    }
}
