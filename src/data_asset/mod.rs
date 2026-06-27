mod tileset;
mod map_data;
mod room;
mod world;
mod sprite;
mod sprite_animation;
mod pal_sprite;
mod sfx;
mod mod_data;
mod font;
mod prop_font;
mod reader;
mod writer;
mod header_def;

use std::{fmt, fs, io};
use std::path::Path;
use std::collections::HashMap;

pub use reader::tokenizer::{Tokenizer, Token, TokenData};
pub use room::RoomTriggerTypeIdent;

pub use tileset::Tileset;
pub use map_data::MapData;
pub use room::{Room, RoomMap, RoomTrigger, RoomTriggerType, RoomItem};
pub use world::{World, WorldRegion};
pub use sprite::Sprite;
pub use pal_sprite::{PalSprite, PalSpriteDepth};
pub use sprite_animation::{SpriteAnimation, SpriteAnimationFrame, SpriteAnimationLoop};
pub use sfx::Sfx;
pub use mod_data::{MOD_PERIOD_TABLE, ModData, ModSample, ModCell};
pub use font::Font;
pub use prop_font::PropFont;
pub use header_def::write_header_def;

pub struct StringLogger {
    log: String,
    print: bool,
}

impl StringLogger {
    pub fn new(print: bool) -> Self {
        StringLogger {
            log: String::new(),
            print,
        }
    }

    //pub fn clear(&mut self) {
    //    self.log.clear();
    //}

    pub fn log<S: AsRef<str>>(&mut self, msg: S) {
        self.log.push_str(msg.as_ref());
        self.log.push('\n');
        if self.print {
            println!("{}", msg.as_ref());
        }
    }

    pub fn read(&self) -> &str {
        &self.log
    }

    /*
    pub fn modify(&mut self) -> &mut String {
        &mut self.log
    }
    */
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, std::hash::Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub w: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct DataAssetId {
    id: u32,
}

impl fmt::Display for DataAssetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataAssetType {
    Tileset,
    MapData,
    Room,
    World,
    Sprite,
    PalSprite,
    SpriteAnimation,
    Sfx,
    ModData,
    Font,
    PropFont,
}

impl DataAssetType {
    pub fn name(&self) -> &'static str {
        match self {
            DataAssetType::Tileset => "tileset",
            DataAssetType::MapData => "map",
            DataAssetType::Room => "room",
            DataAssetType::World => "world",
            DataAssetType::Sprite => "sprite",
            DataAssetType::PalSprite => "pal_sprite",
            DataAssetType::SpriteAnimation => "animation",
            DataAssetType::Sfx => "sfx",
            DataAssetType::ModData => "mod",
            DataAssetType::Font => "font",
            DataAssetType::PropFont => "prop_font",
        }
    }
}

#[derive(std::hash::Hash)]
pub struct DataAsset {
    pub asset_type: DataAssetType,
    pub id: DataAssetId,
    pub name: String,
}

impl DataAsset {
    pub const PATH_SEPARATOR: &str = "/";
    const IDENTIFIER_PATH_SEPARATOR: &str = "__";

    fn new(asset_type: DataAssetType, id: DataAssetId, name: String) -> Self {
        DataAsset {
            asset_type,
            id,
            name,
        }
    }

    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> Self {
        DataAsset {
            asset_type: self.asset_type,
            id: dup_id,
            name: dup_name,
        }
    }

    fn name_to_identifier(name: &str) -> String {
        name.replace(Self::PATH_SEPARATOR, Self::IDENTIFIER_PATH_SEPARATOR)
    }

    fn identifier_to_name(identifier: &str) -> String {
        identifier.replace(Self::IDENTIFIER_PATH_SEPARATOR, Self::PATH_SEPARATOR)
    }
}

pub(crate) trait DuplicableAsset<T> {
    fn duplicate(&self, dup_id: DataAssetId, dup_name: String) -> T;
}

pub trait GenericAsset {
    fn asset(&self) -> &DataAsset;
    fn data_size(&self) -> usize;
}

pub struct AssetList<T> {
    store: HashMap<DataAssetId, T>
}

impl<T> AssetList<T> {
    fn new() -> Self {
        AssetList {
            store: HashMap::new(),
        }
    }

    fn insert(&mut self, id: DataAssetId, asset: T) {
        self.store.insert(id, asset);
    }

    fn remove(&mut self, id: &DataAssetId) -> Option<T> {
        self.store.remove(id)
    }

    pub fn contains(&self, id: &DataAssetId) -> bool {
        self.store.contains_key(id)
    }

    pub fn get(&self, id: &DataAssetId) -> Option<&T> {
        self.store.get(id)
    }

    pub fn get_mut(&mut self, id: &DataAssetId) -> Option<&mut T> {
        self.store.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.store.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.store.values_mut()
    }
}

impl<T: DuplicableAsset<T>> AssetList<T> {
    fn duplicate_asset(&mut self, id: &DataAssetId, dup_name: &str, id_gen: &mut DataAssetIdGenerator) -> Option<DataAssetId> {
        let dup = if let Some(asset) = self.store.get(id) {
            let dup_id = id_gen.gen_id();
            let dup_asset = asset.duplicate(dup_id, dup_name.to_owned());
            Some((dup_id, dup_asset))
        } else {
            None
        };
        if let Some((dup_id, dup_asset)) = dup {
            self.store.insert(dup_id, dup_asset);
            Some(dup_id)
        } else {
            None
        }
    }
}

pub struct AssetIdList {
    store: Vec<DataAssetId>
}

impl AssetIdList {
    fn new() -> Self {
        AssetIdList {
            store: Vec::new(),
        }
    }

    fn push(&mut self, id: DataAssetId) {
        self.store.push(id);
    }

    fn remove_id(&mut self, remove_id: DataAssetId) {
        self.store.retain(|&id| id != remove_id)
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DataAssetId> {
        self.store.iter()
    }

    pub fn get_first(&self) -> Option<DataAssetId> {
        self.store.first().copied()
    }
}

pub struct AssetCollection {
    pub tilesets: AssetList<Tileset>,
    pub maps: AssetList<MapData>,
    pub rooms: AssetList<Room>,
    pub worlds: AssetList<World>,
    pub sprites: AssetList<Sprite>,
    pub pal_sprites: AssetList<PalSprite>,
    pub animations: AssetList<SpriteAnimation>,
    pub sfxs: AssetList<Sfx>,
    pub mods: AssetList<ModData>,
    pub fonts: AssetList<Font>,
    pub prop_fonts: AssetList<PropFont>,
}

impl AssetCollection {
    fn new() -> Self {
        AssetCollection {
            tilesets: AssetList::new(),
            maps: AssetList::new(),
            rooms: AssetList::new(),
            worlds: AssetList::new(),
            sprites: AssetList::new(),
            pal_sprites: AssetList::new(),
            animations: AssetList::new(),
            sfxs: AssetList::new(),
            mods: AssetList::new(),
            fonts: AssetList::new(),
            prop_fonts: AssetList::new(),
        }
    }

    pub fn get_asset(&self, asset_id : DataAssetId) -> Option<&DataAsset> {
        if let Some(v) = self.tilesets.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.maps.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.rooms.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.worlds.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.sprites.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.pal_sprites.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.animations.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.sfxs.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.mods.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.fonts.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.prop_fonts.get(&asset_id) { return Some(&v.asset); }
        None
    }

    /*
    pub fn get_asset_mut(&mut self, asset_id : DataAssetId) -> Option<&mut DataAsset> {
        if let Some(v) = self.tilesets.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.maps.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.rooms.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.worlds.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.sprites.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.pal_sprites.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.animations.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.sfxs.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.mods.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.fonts.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.prop_fonts.get_mut(&asset_id) { return Some(&mut v.asset); }
        None
    }
    */

    pub fn asset_has_dependents(&self, id: DataAssetId) -> bool {
        for map in self.maps.iter() {
            if map.tileset_id == id {
                return true;
            }
        }
        for anim in self.animations.iter() {
            if anim.sprite_id == id {
                return true;
            }
        }
        for room in self.rooms.iter() {
            if room.maps.iter().any(|m| m.map_id == id) {
                return true;
            }
            if room.triggers.iter().any(|e| {
                match e.trigger_type {
                    RoomTriggerType::Trap {..} |
                    RoomTriggerType::Unknown {..} |
                    RoomTriggerType::PlayerSpawn {..} => { false }

                    RoomTriggerType::EnemySpawn { animation_id, .. } => { animation_id == id }
                    RoomTriggerType::Door { room_id, .. } => { room_id == id }
                }
            }) {
                return true;
            }
        }
        for world in self.worlds.iter() {
            for region in world.regions.iter() {
                if region.rooms.contains(&id) {
                    return true;
                }
            }
        }
        false
    }

    pub fn data_size(&self) -> usize {
        let sum = self.tilesets.iter().fold(0, |sum, a| sum + a.data_size());
        let sum = self.maps.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.rooms.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.worlds.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.sprites.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.pal_sprites.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.animations.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.sfxs.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.mods.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.fonts.iter().fold(sum, |sum, a| sum + a.data_size());
        self.prop_fonts.iter().fold(sum, |sum, a| sum + a.data_size())
    }
}

pub struct AssetIdCollection {
    pub tilesets: AssetIdList,
    pub maps: AssetIdList,
    pub rooms: AssetIdList,
    pub worlds: AssetIdList,
    pub sprites: AssetIdList,
    pub pal_sprites: AssetIdList,
    pub animations: AssetIdList,
    pub sfxs: AssetIdList,
    pub mods: AssetIdList,
    pub fonts: AssetIdList,
    pub prop_fonts: AssetIdList,
}

impl AssetIdCollection {
    fn new() -> Self {
        AssetIdCollection {
            tilesets: AssetIdList::new(),
            maps: AssetIdList::new(),
            rooms: AssetIdList::new(),
            worlds: AssetIdList::new(),
            sprites: AssetIdList::new(),
            pal_sprites: AssetIdList::new(),
            animations: AssetIdList::new(),
            sfxs: AssetIdList::new(),
            mods: AssetIdList::new(),
            fonts: AssetIdList::new(),
            prop_fonts: AssetIdList::new(),
        }
    }

    pub fn ids_of_type(&self, asset_type: DataAssetType) -> impl Iterator<Item = &DataAssetId> {
        match asset_type {
            DataAssetType::Tileset => self.tilesets.iter(),
            DataAssetType::MapData => self.maps.iter(),
            DataAssetType::Room => self.rooms.iter(),
            DataAssetType::World => self.worlds.iter(),
            DataAssetType::Sprite => self.sprites.iter(),
            DataAssetType::PalSprite => self.pal_sprites.iter(),
            DataAssetType::SpriteAnimation => self.animations.iter(),
            DataAssetType::Sfx => self.sfxs.iter(),
            DataAssetType::ModData => self.mods.iter(),
            DataAssetType::Font => self.fonts.iter(),
            DataAssetType::PropFont => self.prop_fonts.iter(),
        }
    }
}

struct DataAssetIdGenerator {
    next_id: u32,
}

impl DataAssetIdGenerator {
    fn new() -> Self {
        DataAssetIdGenerator {
            next_id: 0,
        }
    }

    fn gen_id(&mut self) -> DataAssetId {
        let id = self.next_id;
        self.next_id += 1;
        DataAssetId { id }
    }
}

pub struct DataAssetStore {
    id_generator: DataAssetIdGenerator,
    pub vga_bits_per_pixel: u8,
    pub vga_sync_bits: u8,
    pub project_prefix: String,
    pub assets: AssetCollection,
    pub asset_ids: AssetIdCollection,
}

impl DataAssetStore {
    pub const VERSION: u32 = 6;
    pub const VERSION_DATE: &str = "2026-06-25";

    pub fn new() -> Self {
        DataAssetStore {
            id_generator: DataAssetIdGenerator::new(),
            vga_bits_per_pixel: 8,
            vga_sync_bits: 0xc0,
            project_prefix: String::from("PROJECT"),
            assets: AssetCollection::new(),
            asset_ids: AssetIdCollection::new(),
        }
    }

    pub fn read_file<P: AsRef<Path>>(filename: P, logger: &mut StringLogger) -> Result<Self, io::Error> {
        fs::read_to_string(filename).and_then(|file_content| {
            reader::ProjectDataReader::read_from_string(&file_content, logger)
        })
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, filename: P, logger: &mut StringLogger) -> Result<(), io::Error> {
        writer::ProjectDataWriter::write_to_file(filename, self, logger)
    }

    fn gen_id(&mut self) -> DataAssetId {
        self.id_generator.gen_id()
    }

    pub fn num_assets(&self) -> usize {
        self.assets.tilesets.store.len() +
            self.assets.maps.store.len() +
            self.assets.rooms.store.len() +
            self.assets.worlds.store.len() +
            self.assets.sprites.store.len() +
            self.assets.pal_sprites.store.len() +
            self.assets.animations.store.len() +
            self.assets.sfxs.store.len() +
            self.assets.mods.store.len() +
            self.assets.fonts.store.len() +
            self.assets.prop_fonts.store.len()
    }

    pub fn remove_asset(&mut self, id: DataAssetId) -> Option<DataAsset> {
        if let Some(v) = self.assets.tilesets.remove(&id) { self.asset_ids.tilesets.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.maps.remove(&id) { self.asset_ids.maps.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.rooms.remove(&id) { self.asset_ids.rooms.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.worlds.remove(&id) { self.asset_ids.worlds.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.sprites.remove(&id) { self.asset_ids.sprites.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.pal_sprites.remove(&id) { self.asset_ids.pal_sprites.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.animations.remove(&id) { self.asset_ids.animations.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.sfxs.remove(&id) { self.asset_ids.sfxs.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.mods.remove(&id) { self.asset_ids.mods.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.fonts.remove(&id) { self.asset_ids.fonts.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.prop_fonts.remove(&id) { self.asset_ids.prop_fonts.remove_id(id); return Some(v.asset); }
        None
    }

    pub fn duplicate_asset(&mut self, id: DataAssetId, dup_name: &str) -> Option<DataAssetId> {
        if let Some(dup_id) = self.assets.tilesets.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.tilesets.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.maps.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.maps.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.rooms.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.rooms.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.worlds.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.worlds.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.sprites.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.sprites.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.pal_sprites.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.pal_sprites.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.animations.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.animations.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.sfxs.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.sfxs.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.mods.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.mods.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.fonts.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.fonts.push(dup_id);
            return Some(dup_id);
        }
        if let Some(dup_id) = self.assets.prop_fonts.duplicate_asset(&id, dup_name, &mut self.id_generator) {
            self.asset_ids.prop_fonts.push(dup_id);
            return Some(dup_id);
        }
        None
    }

    pub fn add_tileset(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.tilesets.push(id);
        self.assets.tilesets.insert(id, Tileset::new(id, name));
        Some(id)
    }

    pub fn add_map(&mut self, name: String, tileset_id: DataAssetId) -> Option<DataAssetId> {
        if ! self.assets.tilesets.contains(&tileset_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.maps.push(id);
        self.assets.maps.insert(id, MapData::new(id, name, tileset_id));
        Some(id)
    }

    pub fn add_room(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.rooms.push(id);
        self.assets.rooms.insert(id, Room::new(id, name));
        Some(id)
    }

    pub fn add_world(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.worlds.push(id);
        self.assets.worlds.insert(id, World::new(id, name));
        Some(id)
    }

    pub fn add_sprite(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sprites.push(id);
        self.assets.sprites.insert(id, Sprite::new(id, name));
        Some(id)
    }

    pub fn add_pal_sprite(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.pal_sprites.push(id);
        self.assets.pal_sprites.insert(id, PalSprite::new(id, name));
        Some(id)
    }

    pub fn add_animation(&mut self, name: String, sprite_id: DataAssetId) -> Option<DataAssetId> {
        if ! self.assets.sprites.contains(&sprite_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.animations.push(id);
        self.assets.animations.insert(id, SpriteAnimation::new(id, name, sprite_id));
        Some(id)
    }

    pub fn add_sfx(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sfxs.push(id);
        self.assets.sfxs.insert(id, Sfx::new(id, name));
        Some(id)
    }

    pub fn add_mod(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.mods.push(id);
        self.assets.mods.insert(id, ModData::new(id, name));
        Some(id)
    }

    pub fn add_font(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.fonts.push(id);
        self.assets.fonts.insert(id, Font::new(id, name));
        Some(id)
    }

    pub fn add_prop_font(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.prop_fonts.push(id);
        self.assets.prop_fonts.insert(id, PropFont::new(id, name));
        Some(id)
    }
}
