pub mod tileset;
pub mod map_data;
pub mod room;
pub mod sprite;
pub mod sprite_animation;
pub mod sfx;
pub mod mod_data;
pub mod font;
pub mod prop_font;
pub mod reader;

use std::fmt;
use std::collections::HashMap;

pub use tileset::Tileset;
pub use map_data::MapData;
pub use room::{Room, RoomMap, RoomEntity, RoomTrigger};
pub use sprite::Sprite;
pub use sprite_animation::{SpriteAnimation, SpriteAnimationFrame};
pub use sfx::Sfx;
pub use mod_data::{ModData, ModSample, ModCell};
pub use font::Font;
pub use prop_font::PropFont;

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

    pub fn log(&mut self, msg: &str) {
        self.log.push_str(msg);
        self.log.push('\n');
        if self.print {
            println!("{}", msg);
        }
    }

    /*
    pub fn read(&self) -> &str {
        &self.log
    }
    */

    pub fn modify(&mut self) -> &mut String {
        &mut self.log
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    x: i32,
    y: i32,
    h: i32,
    w: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DataAssetId {
    id: u32,
}

impl fmt::Display for DataAssetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataAssetType {
    Tileset,
    MapData,
    Room,
    Sprite,
    SpriteAnimation,
    Sfx,
    ModData,
    Font,
    PropFont,
}

pub struct DataAsset {
    //pub asset_type: DataAssetType,
    pub id: DataAssetId,
    pub name: String,
}

impl DataAsset {
    fn new(id: DataAssetId, name: String) -> Self {
        DataAsset {
            id,
            name,
        }
    }
}

pub trait GenericAsset {
    //fn asset(&self) -> &DataAsset;
    fn data_size(&self) -> usize;
}

pub trait ImageCollectionAsset {
    fn asset_id(&self) -> DataAssetId;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn num_items(&self) -> u32;
    fn data(&self) -> &[u8];
}

pub fn image_pixels_prop_font_to_u8(bits: &[u8], widths: &[u8], height: u32, num_items: u32, offsets: &[u16]) -> Vec<u8> {
    let height = height as usize;
    let num_items = num_items as usize;
    let max_width = 2 * height;
    let mut pixels = vec![0x0c; max_width * height * num_items];
    if widths.len() != num_items || offsets.len() != widths.len() { return pixels; }
    for ch in 0..num_items {
        let offset = offsets[ch] as usize;
        let width = widths[ch] as usize;
        let stride = width.div_ceil(8);
        for y in 0..height {
            for x in 0..stride {
                let block = bits.get(offset + y*stride + x).map_or(0, |&v| v);
                for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                    if block & (1 << ix) != 0 && (x*8 + ix) < max_width {
                        pixels[((ch * height) + y) * max_width + x*8 + ix] = 0;
                    }
                }
            }
        }
    }
    pixels
}

pub fn image_pixels_font_to_u8(bits: &[u8], width: u32, height: u32, num_items: u32) -> Vec<u8> {
    let stride = width.div_ceil(8) as usize;
    let mut pixels = vec![0x0cu8; (width * height * num_items) as usize];
    for y in 0..(height * num_items) as usize {
        for x in 0..stride {
            let block = bits.get(y*stride + x).map_or(0, |&v| v);
            for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                if block & (1 << ix) != 0 {
                    pixels[y * width as usize + x*8 + ix] = 0;
                }
            }
        }
    }
    pixels
}

pub fn image_pixels_u32_to_u8(data: &[u32], width: u32, height: u32, num_items: u32) -> Vec<u8> {
    let stride = width.div_ceil(4) as usize;
    let mut pixels = Vec::<u8>::with_capacity((width * height * num_items) as usize);
    for y in 0 .. (height * num_items) as usize {
        for x in 0..stride {
            let quad = data[y*stride + x];
            if x < stride-1 || width.is_multiple_of(4) {
                pixels.push((quad         & 0xff) as u8);
                pixels.push(((quad >>  8) & 0xff) as u8);
                pixels.push(((quad >> 16) & 0xff) as u8);
                pixels.push(((quad >> 24) & 0xff) as u8);
            } else {
                match width % 4 {
                    1 => {
                        pixels.push((quad         & 0xff) as u8);
                    },
                    2 => {
                        pixels.push((quad         & 0xff) as u8);
                        pixels.push(((quad >>  8) & 0xff) as u8);
                    },
                    3 => {
                        pixels.push((quad         & 0xff) as u8);
                        pixels.push(((quad >>  8) & 0xff) as u8);
                        pixels.push(((quad >> 16) & 0xff) as u8);
                    },
                    _ => {},
                }
            }
        }
    }
    pixels
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

    //pub fn get(&self, index: usize) -> Option<&DataAssetId> {
    //    self.store.get(index)
    //}

    //pub fn len(&self) -> usize {
    //    self.store.len()
    //}

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
    pub sprites: AssetList<Sprite>,
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
            sprites: AssetList::new(),
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
        if let Some(v) = self.sprites.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.animations.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.sfxs.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.mods.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.fonts.get(&asset_id) { return Some(&v.asset); }
        if let Some(v) = self.prop_fonts.get(&asset_id) { return Some(&v.asset); }
        None
    }

    pub fn get_asset_mut(&mut self, asset_id : DataAssetId) -> Option<&mut DataAsset> {
        if let Some(v) = self.tilesets.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.maps.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.rooms.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.sprites.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.animations.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.sfxs.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.mods.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.fonts.get_mut(&asset_id) { return Some(&mut v.asset); }
        if let Some(v) = self.prop_fonts.get_mut(&asset_id) { return Some(&mut v.asset); }
        None
    }

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
            if room.entities.iter().any(|e| e.animation_id == id) {
                return true;
            }
        }
        false
    }

    pub fn data_size(&self) -> usize {
        let sum = self.tilesets.iter().fold(0, |sum, a| sum + a.data_size());
        let sum = self.maps.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.rooms.iter().fold(sum, |sum, a| sum + a.data_size());
        let sum = self.sprites.iter().fold(sum, |sum, a| sum + a.data_size());
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
    pub sprites: AssetIdList,
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
            sprites: AssetIdList::new(),
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
            DataAssetType::Sprite => self.sprites.iter(),
            DataAssetType::SpriteAnimation => self.animations.iter(),
            DataAssetType::Sfx => self.sfxs.iter(),
            DataAssetType::ModData => self.mods.iter(),
            DataAssetType::Font => self.fonts.iter(),
            DataAssetType::PropFont => self.prop_fonts.iter(),
        }
    }

}

pub struct DataAssetStore {
    next_id: u32,

    pub vga_sync_bits: u8,
    pub project_prefix: String,
    pub assets: AssetCollection,
    pub asset_ids: AssetIdCollection,
}

impl DataAssetStore {

    pub fn new() -> Self {
        DataAssetStore {
            next_id: 0,
            vga_sync_bits: 0,
            project_prefix: String::new(),
            assets: AssetCollection::new(),
            asset_ids: AssetIdCollection::new(),
        }
    }

    fn gen_id(&mut self) -> DataAssetId {
        let id = self.next_id;
        self.next_id += 1;
        DataAssetId { id }
    }

    pub fn remove_asset(&mut self, id: DataAssetId) -> Option<DataAsset> {
        if let Some(v) = self.assets.tilesets.remove(&id) { self.asset_ids.tilesets.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.maps.remove(&id) { self.asset_ids.maps.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.rooms.remove(&id) { self.asset_ids.rooms.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.sprites.remove(&id) { self.asset_ids.sprites.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.animations.remove(&id) { self.asset_ids.animations.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.sfxs.remove(&id) { self.asset_ids.sfxs.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.mods.remove(&id) { self.asset_ids.mods.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.fonts.remove(&id) { self.asset_ids.fonts.remove_id(id); return Some(v.asset); }
        if let Some(v) = self.assets.prop_fonts.remove(&id) { self.asset_ids.prop_fonts.remove_id(id); return Some(v.asset); }
        None
    }

    pub fn add_tileset(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.tilesets.push(id);
        self.assets.tilesets.insert(id, Tileset::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_tileset_from(&mut self, name: String, data: tileset::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.tilesets.push(id);
        self.assets.tilesets.insert(id, Tileset::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_map(&mut self, name: String, tileset_id: DataAssetId) -> Option<DataAssetId> {
        if ! self.assets.tilesets.contains(&tileset_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.maps.push(id);
        self.assets.maps.insert(id, MapData::new(DataAsset::new(id, name), tileset_id));
        Some(id)
    }

    pub fn add_map_from(&mut self, name: String, data: map_data::CreationData) -> Option<DataAssetId> {
        if ! self.assets.tilesets.contains(&data.tileset_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.maps.push(id);
        self.assets.maps.insert(id, MapData::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_room(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.rooms.push(id);
        self.assets.rooms.insert(id, Room::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_room_from(&mut self, name: String, data: room::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.rooms.push(id);
        self.assets.rooms.insert(id, Room::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_sprite(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sprites.push(id);
        self.assets.sprites.insert(id, Sprite::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_sprite_from(&mut self, name: String, data: sprite::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sprites.push(id);
        self.assets.sprites.insert(id, Sprite::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_animation(&mut self, name: String, sprite_id: DataAssetId) -> Option<DataAssetId> {
        if ! self.assets.sprites.contains(&sprite_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.animations.push(id);
        self.assets.animations.insert(id, SpriteAnimation::new(DataAsset::new(id, name), sprite_id));
        Some(id)
    }

    pub fn add_animation_from(&mut self, name: String, data: sprite_animation::CreationData) -> Option<DataAssetId> {
        if ! self.assets.sprites.contains(&data.sprite_id) {
            return None;
        }
        let id = self.gen_id();
        self.asset_ids.animations.push(id);
        self.assets.animations.insert(id, SpriteAnimation::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_sfx(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sfxs.push(id);
        self.assets.sfxs.insert(id, Sfx::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_sfx_from(&mut self, name: String, data: sfx::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.sfxs.push(id);
        self.assets.sfxs.insert(id, Sfx::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_mod(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.mods.push(id);
        self.assets.mods.insert(id, ModData::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_mod_from(&mut self, name: String, data: mod_data::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.mods.push(id);
        self.assets.mods.insert(id, ModData::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_font(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.fonts.push(id);
        self.assets.fonts.insert(id, Font::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_font_from(&mut self, name: String, data: font::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.fonts.push(id);
        self.assets.fonts.insert(id, Font::from_data(DataAsset::new(id, name), data));
        Some(id)
    }

    pub fn add_prop_font(&mut self, name: String) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.prop_fonts.push(id);
        self.assets.prop_fonts.insert(id, PropFont::new(DataAsset::new(id, name)));
        Some(id)
    }

    pub fn add_prop_font_from(&mut self, name: String, data: prop_font::CreationData) -> Option<DataAssetId> {
        let id = self.gen_id();
        self.asset_ids.prop_fonts.push(id);
        self.assets.prop_fonts.insert(id, PropFont::from_data(DataAsset::new(id, name), data));
        Some(id)
    }
}
