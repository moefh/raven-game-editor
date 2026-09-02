mod tileset;
mod tile_animation;
mod map_data;
mod room;
mod world;
mod sprite;
mod pal_sprite;
mod sprite_animation;
mod mod_data;
mod sfx;
mod font;
mod prop_font;

pub use tileset::{*};
pub use tile_animation::{*};
pub use map_data::{*};
pub use room::{*};
pub use world::{*};
pub use sprite::{*};
pub use pal_sprite::{*};
pub use sprite_animation::{*};
pub use mod_data::{*};
pub use sfx::{*};
pub use font::{*};
pub use prop_font::{*};

use crate::data_asset::{
    DataAssetStore,
    DataAssetType,
    Tileset,
    TileAnimation,
    MapData,
    Room,
    World,
    Sprite,
    SpriteAnimation,
    PalSprite,
    ModData,
    Sfx,
    Font,
    PropFont,
};

pub fn get_tileset_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Tileset> {
    store.assets.tilesets.iter().find(|asset| asset.asset.name == name)
}
pub fn get_tile_animation_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a TileAnimation> {
    store.assets.tile_anims.iter().find(|asset| asset.asset.name == name)
}
pub fn get_map_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a MapData> {
    store.assets.maps.iter().find(|asset| asset.asset.name == name)
}
pub fn get_room_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Room> {
    store.assets.rooms.iter().find(|asset| asset.asset.name == name)
}
pub fn get_world_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a World> {
    store.assets.worlds.iter().find(|asset| asset.asset.name == name)
}
pub fn get_sprite_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Sprite> {
    store.assets.sprites.iter().find(|asset| asset.asset.name == name)
}
pub fn get_sprite_animation_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a SpriteAnimation> {
    store.assets.animations.iter().find(|asset| asset.asset.name == name)
}
pub fn get_pal_sprite_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a PalSprite> {
    store.assets.pal_sprites.iter().find(|asset| asset.asset.name == name)
}
pub fn get_mod_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a ModData> {
    store.assets.mods.iter().find(|asset| asset.asset.name == name)
}
pub fn get_sfx_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Sfx> {
    store.assets.sfxs.iter().find(|asset| asset.asset.name == name)
}
pub fn get_font_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a Font> {
    store.assets.fonts.iter().find(|asset| asset.asset.name == name)
}
pub fn get_prop_font_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a PropFont> {
    store.assets.prop_fonts.iter().find(|asset| asset.asset.name == name)
}

pub struct BaseProjectDiff {
    pub cur_data_size: usize,
    pub other_data_size: usize,
}

impl BaseProjectDiff {
    fn new() -> Self {
        BaseProjectDiff {
            cur_data_size: 0,
            other_data_size: 0,
        }
    }

    pub fn clear(&mut self) {
        self.cur_data_size = 0;
        self.other_data_size = 0;
    }

    pub fn compare(&mut self, store: &DataAssetStore, other: &DataAssetStore) {
        self.cur_data_size = store.assets.data_size();
        self.other_data_size = other.assets.data_size();
    }
}

pub struct ProjectComparer {
    pub base_project: BaseProjectDiff,
    pub tilesets: TilesetListDiff,
    pub tile_anims: TileAnimationListDiff,
    pub maps: MapListDiff,
    pub rooms: RoomListDiff,
    pub worlds: WorldListDiff,
    pub sprites: SpriteListDiff,
    pub pal_sprites: PalSpriteListDiff,
    pub animations: SpriteAnimationListDiff,
    pub mods: ModListDiff,
    pub sfxs: SfxListDiff,
    pub fonts: FontListDiff,
    pub prop_fonts: PropFontListDiff,
}

impl ProjectComparer {
    pub fn new() -> Self {
        ProjectComparer {
            base_project: BaseProjectDiff::new(),
            tilesets: TilesetListDiff::new(),
            tile_anims: TileAnimationListDiff::new(),
            maps: MapListDiff::new(),
            rooms: RoomListDiff::new(),
            worlds: WorldListDiff::new(),
            sprites: SpriteListDiff::new(),
            pal_sprites: PalSpriteListDiff::new(),
            animations: SpriteAnimationListDiff::new(),
            mods: ModListDiff::new(),
            sfxs: SfxListDiff::new(),
            fonts: FontListDiff::new(),
            prop_fonts: PropFontListDiff::new(),
        }
    }

    pub fn has_any_asset_differences(&self) -> bool {
        ! (self.tilesets.is_empty() &&
            self.tile_anims.is_empty() &&
            self.maps.is_empty() &&
            self.rooms.is_empty() &&
            self.worlds.is_empty() &&
            self.sprites.is_empty() &&
            self.animations.is_empty() &&
            self.pal_sprites.is_empty() &&
            self.mods.is_empty() &&
            self.sfxs.is_empty() &&
            self.fonts.is_empty() &&
            self.prop_fonts.is_empty())
    }

    pub fn has_asset_differences(&self, asset_type: DataAssetType) -> bool {
        match asset_type {
            DataAssetType::Tileset => { ! self.tilesets.is_empty() }
            DataAssetType::TileAnimation => { ! self.tile_anims.is_empty() }
            DataAssetType::MapData => { ! self.maps.is_empty() }
            DataAssetType::Room => { ! self.rooms.is_empty() }
            DataAssetType::World => { ! self.worlds.is_empty() }
            DataAssetType::Sprite => { ! self.sprites.is_empty() }
            DataAssetType::SpriteAnimation => { ! self.animations.is_empty() }
            DataAssetType::PalSprite => { ! self.pal_sprites.is_empty() }
            DataAssetType::ModData => { ! self.mods.is_empty() }
            DataAssetType::Sfx => { ! self.sfxs.is_empty() }
            DataAssetType::Font => { ! self.fonts.is_empty() }
            DataAssetType::PropFont => { ! self.prop_fonts.is_empty() }
        }
    }

    pub fn clear(&mut self) {
        self.base_project.clear();
        self.tilesets.clear();
        self.tile_anims.clear();
        self.maps.clear();
        self.rooms.clear();
        self.worlds.clear();
        self.sprites.clear();
        self.pal_sprites.clear();
        self.animations.clear();
        self.mods.clear();
        self.sfxs.clear();
        self.fonts.clear();
        self.prop_fonts.clear();
    }

    pub fn run(&mut self, cur_store: &DataAssetStore, other_store: &DataAssetStore) {
        self.base_project.compare(cur_store, other_store);
        self.tilesets.compare(cur_store, other_store);
        self.tile_anims.compare(cur_store, other_store);
        self.maps.compare(cur_store, other_store);
        self.rooms.compare(cur_store, other_store);
        self.worlds.compare(cur_store, other_store);
        self.sprites.compare(cur_store, other_store);
        self.pal_sprites.compare(cur_store, other_store);
        self.animations.compare(cur_store, other_store);
        self.mods.compare(cur_store, other_store);
        self.sfxs.compare(cur_store, other_store);
        self.fonts.compare(cur_store, other_store);
        self.prop_fonts.compare(cur_store, other_store);
    }
}
