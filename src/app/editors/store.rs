use std::collections::{
    HashMap,
    HashSet,
};

use crate::data_asset::{
    DataAssetId,
    DataAssetType,
    DataAssetStore,
    AssetList,
    Room,
};
use super::{
    AssetEditorBase,
    TilesetEditor,
    MapDataEditor,
    RoomEditor,
    WorldEditor,
    SpriteEditor,
    PalSpriteEditor,
    SpriteAnimationEditor,
    SfxEditor,
    ModDataEditor,
    FontEditor,
    PropFontEditor,
};

pub struct EditorStore {
    pub egui_id_to_asset_id: HashMap<egui::Id, DataAssetId>,
    pub tilesets: HashMap<DataAssetId, TilesetEditor>,
    pub maps: HashMap<DataAssetId, MapDataEditor>,
    pub rooms: HashMap<DataAssetId, RoomEditor>,
    pub worlds: HashMap<DataAssetId, WorldEditor>,
    pub sprites: HashMap<DataAssetId, SpriteEditor>,
    pub pal_sprites: HashMap<DataAssetId, PalSpriteEditor>,
    pub animations: HashMap<DataAssetId, SpriteAnimationEditor>,
    pub sfxs: HashMap<DataAssetId, SfxEditor>,
    pub mods: HashMap<DataAssetId, ModDataEditor>,
    pub fonts: HashMap<DataAssetId, FontEditor>,
    pub prop_fonts: HashMap<DataAssetId, PropFontEditor>,
    pub room_names: HashMap<DataAssetId, String>,
}

impl EditorStore {
    pub fn new() -> Self {
        EditorStore {
            egui_id_to_asset_id: HashMap::new(),
            tilesets: HashMap::new(),
            maps: HashMap::new(),
            rooms: HashMap::new(),
            worlds: HashMap::new(),
            sprites: HashMap::new(),
            pal_sprites: HashMap::new(),
            animations: HashMap::new(),
            sfxs: HashMap::new(),
            mods: HashMap::new(),
            fonts: HashMap::new(),
            prop_fonts: HashMap::new(),
            room_names: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.egui_id_to_asset_id.clear();
        self.tilesets.clear();
        self.maps.clear();
        self.rooms.clear();
        self.worlds.clear();
        self.sprites.clear();
        self.pal_sprites.clear();
        self.animations.clear();
        self.sfxs.clear();
        self.mods.clear();
        self.fonts.clear();
        self.prop_fonts.clear();
        self.room_names.clear();
    }

    pub fn create_editors_for_new_store(&mut self, store: &DataAssetStore) {
        for id in store.asset_ids.tilesets.iter().copied() { self.add_tileset(id); }
        for id in store.asset_ids.maps.iter().copied() { self.add_map(id); }
        for id in store.asset_ids.rooms.iter().copied() { self.add_room(id); }
        for id in store.asset_ids.worlds.iter().copied() { self.add_world(id); }
        for id in store.asset_ids.sprites.iter().copied() { self.add_sprite(id); }
        for id in store.asset_ids.pal_sprites.iter().copied() { self.add_pal_sprite(id); }
        for id in store.asset_ids.animations.iter().copied() { self.add_animation(id); }
        for id in store.asset_ids.sfxs.iter().copied() { self.add_sfx(id); }
        for id in store.asset_ids.mods.iter().copied() { self.add_mod(id); }
        for id in store.asset_ids.fonts.iter().copied() { self.add_font(id); }
        for id in store.asset_ids.prop_fonts.iter().copied() { self.add_prop_font(id); }
        self.clear_dirty_flags(store);
    }

    pub fn get_editor(&self, id: DataAssetId) -> Option<&AssetEditorBase> {
        if let Some(editor) = self.tilesets.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.maps.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.rooms.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.worlds.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.sprites.get(&id) { return Some(&editor.base); }
        if let Some(editor) = self.pal_sprites.get(&id) { return Some(&editor.base); }
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
        if let Some(editor) = self.worlds.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.sprites.get_mut(&id) { return Some(&mut editor.base); }
        if let Some(editor) = self.pal_sprites.get_mut(&id) { return Some(&mut editor.base); }
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
        if let Some(editor) = self.worlds.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.sprites.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.pal_sprites.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.animations.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.sfxs.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.mods.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        if let Some(editor) = self.prop_fonts.remove(&id) { self.egui_id_to_asset_id.remove(&editor.base.egui_id); return true; }
        false
    }

    pub fn add_asset(&mut self, id: DataAssetId, asset_type: DataAssetType) -> egui::Id {
        match asset_type {
            DataAssetType::Tileset => self.add_tileset(id),
            DataAssetType::MapData => self.add_map(id),
            DataAssetType::Room => self.add_room(id),
            DataAssetType::World => self.add_world(id),
            DataAssetType::Sprite => self.add_sprite(id),
            DataAssetType::PalSprite => self.add_pal_sprite(id),
            DataAssetType::SpriteAnimation => self.add_animation(id),
            DataAssetType::Sfx => self.add_sfx(id),
            DataAssetType::ModData => self.add_mod(id),
            DataAssetType::Font => self.add_font(id),
            DataAssetType::PropFont => self.add_prop_font(id),
        }
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

    pub fn add_world(&mut self, id: DataAssetId) -> egui::Id {
        let editor = WorldEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.worlds.insert(id, editor);
        egui_id
    }

    pub fn add_sprite(&mut self, id: DataAssetId) -> egui::Id {
        let editor = SpriteEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.sprites.insert(id, editor);
        egui_id
    }

    pub fn add_pal_sprite(&mut self, id: DataAssetId) -> egui::Id {
        let editor = PalSpriteEditor::new(id, false);
        let egui_id = editor.base.egui_id;
        self.egui_id_to_asset_id.insert(egui_id, editor.base.id);
        self.pal_sprites.insert(id, editor);
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

    pub fn update_dirty_flags(&mut self, store: &DataAssetStore) {
        for tileset in store.assets.tilesets.iter() {
            if let Some(editor) = self.tilesets.get_mut(&tileset.asset.id) { editor.base.update_dirty_flag(tileset); }
        }
        for map in store.assets.maps.iter() {
            if let Some(editor) = self.maps.get_mut(&map.asset.id) { editor.base.update_dirty_flag(map); }
        }
        for room in store.assets.rooms.iter() {
            if let Some(editor) = self.rooms.get_mut(&room.asset.id) { editor.base.update_dirty_flag(room); }
        }
        for world in store.assets.worlds.iter() {
            if let Some(editor) = self.worlds.get_mut(&world.asset.id) { editor.base.update_dirty_flag(world); }
        }
        for sprite in store.assets.sprites.iter() {
            if let Some(editor) = self.sprites.get_mut(&sprite.asset.id) { editor.base.update_dirty_flag(sprite); }
        }
        for pal_sprite in store.assets.pal_sprites.iter() {
            if let Some(editor) = self.pal_sprites.get_mut(&pal_sprite.asset.id) { editor.base.update_dirty_flag(pal_sprite); }
        }
        for anim in store.assets.animations.iter() {
            if let Some(editor) = self.animations.get_mut(&anim.asset.id) { editor.base.update_dirty_flag(anim); }
        }
        for sfx in store.assets.sfxs.iter() {
            if let Some(editor) = self.sfxs.get_mut(&sfx.asset.id) { editor.base.update_dirty_flag(sfx); }
        }
        for mod_data in store.assets.mods.iter() {
            if let Some(editor) = self.mods.get_mut(&mod_data.asset.id) { editor.base.update_dirty_flag(mod_data); }
        }
        for font in store.assets.fonts.iter() {
            if let Some(editor) = self.fonts.get_mut(&font.asset.id) { editor.base.update_dirty_flag(font); }
        }
        for pfont in store.assets.prop_fonts.iter() {
            if let Some(editor) = self.prop_fonts.get_mut(&pfont.asset.id) { editor.base.update_dirty_flag(pfont); }
        }
    }

    pub fn clear_dirty_flags(&mut self, store: &DataAssetStore) {
        for tileset in store.assets.tilesets.iter() {
            if let Some(editor) = self.tilesets.get_mut(&tileset.asset.id) { editor.base.clear_dirty_flag(tileset); }
        }
        for map in store.assets.maps.iter() {
            if let Some(editor) = self.maps.get_mut(&map.asset.id) { editor.base.clear_dirty_flag(map); }
        }
        for room in store.assets.rooms.iter() {
            if let Some(editor) = self.rooms.get_mut(&room.asset.id) { editor.base.clear_dirty_flag(room); }
        }
        for world in store.assets.worlds.iter() {
            if let Some(editor) = self.worlds.get_mut(&world.asset.id) { editor.base.clear_dirty_flag(world); }
        }
        for sprite in store.assets.sprites.iter() {
            if let Some(editor) = self.sprites.get_mut(&sprite.asset.id) { editor.base.clear_dirty_flag(sprite); }
        }
        for pal_sprite in store.assets.pal_sprites.iter() {
            if let Some(editor) = self.pal_sprites.get_mut(&pal_sprite.asset.id) { editor.base.clear_dirty_flag(pal_sprite); }
        }
        for anim in store.assets.animations.iter() {
            if let Some(editor) = self.animations.get_mut(&anim.asset.id) { editor.base.clear_dirty_flag(anim); }
        }
        for sfx in store.assets.sfxs.iter() {
            if let Some(editor) = self.sfxs.get_mut(&sfx.asset.id) { editor.base.clear_dirty_flag(sfx); }
        }
        for mod_data in store.assets.mods.iter() {
            if let Some(editor) = self.mods.get_mut(&mod_data.asset.id) { editor.base.clear_dirty_flag(mod_data); }
        }
        for font in store.assets.fonts.iter() {
            if let Some(editor) = self.fonts.get_mut(&font.asset.id) { editor.base.clear_dirty_flag(font); }
        }
        for pfont in store.assets.prop_fonts.iter() {
            if let Some(editor) = self.prop_fonts.get_mut(&pfont.asset.id) { editor.base.clear_dirty_flag(pfont); }
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.iter().any(|e| e.is_dirty())
    }

    pub fn iter(&self) -> impl Iterator<Item = &AssetEditorBase> {
        self.tilesets.values().map(|e| &e.base)
            .chain(self.maps.values().map(|e| &e.base))
            .chain(self.rooms.values().map(|e| &e.base))
            .chain(self.worlds.values().map(|e| &e.base))
            .chain(self.sprites.values().map(|e| &e.base))
            .chain(self.pal_sprites.values().map(|e| &e.base))
            .chain(self.animations.values().map(|e| &e.base))
            .chain(self.sfxs.values().map(|e| &e.base))
            .chain(self.mods.values().map(|e| &e.base))
            .chain(self.fonts.values().map(|e| &e.base))
            .chain(self.prop_fonts.values().map(|e| &e.base))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AssetEditorBase> {
        self.tilesets.values_mut().map(|e| &mut e.base)
            .chain(self.maps.values_mut().map(|e| &mut e.base))
            .chain(self.rooms.values_mut().map(|e| &mut e.base))
            .chain(self.worlds.values_mut().map(|e| &mut e.base))
            .chain(self.sprites.values_mut().map(|e| &mut e.base))
            .chain(self.pal_sprites.values_mut().map(|e| &mut e.base))
            .chain(self.animations.values_mut().map(|e| &mut e.base))
            .chain(self.sfxs.values_mut().map(|e| &mut e.base))
            .chain(self.mods.values_mut().map(|e| &mut e.base))
            .chain(self.fonts.values_mut().map(|e| &mut e.base))
            .chain(self.prop_fonts.values_mut().map(|e| &mut e.base))
    }

    pub fn get_open_ids(&mut self) -> impl Iterator<Item = egui::Id> {
        self.iter().filter_map(|e| { if e.open { Some(e.egui_id) } else { None }})
    }

    pub fn refresh_room_names(&mut self, rooms: &AssetList<Room>) {
        let mut seen_ids = HashSet::new();
        let mut remove_ids = HashSet::new();

        // correct changed names
        for (room_id, room_name) in self.room_names.iter_mut() {
            seen_ids.insert(*room_id);
            if let Some(room) = rooms.get(room_id) {
                if *room_name != room.asset.name {
                    room_name.clear();
                    room_name.push_str(&room.asset.name);
                }
            } else {
                remove_ids.insert(*room_id);
            }
        }

        // add new rooms
        for room in rooms.iter() {
            if ! seen_ids.contains(&room.asset.id) {
                self.room_names.insert(room.asset.id, room.asset.name.clone());
            }
        }

        // remove gone rooms
        for room_id in remove_ids.iter() {
            self.room_names.remove(room_id);
        }
    }
}
