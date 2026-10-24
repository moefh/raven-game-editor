use crate::misc::{
    STATIC_IMAGES,
};
use crate::data_asset::{
    AssetList,
    AssetIdCollection,
    Tileset,
    MapData,
    Room,
    RoomParallax,
    RoomEnemyType,
    RoomTrigger,
    RoomTriggerType,
    RoomEntityDirection,
    RoomTriggerMapLayer,
};
use crate::image::{
    TextureSlot,
    ImageCollection,
};

use super::{
    WindowContext,
    MapTileFixer,
};
use super::super::widgets::{
    TILE_SIZE,
    FULL_UV,
};
use super::super::super::widgets::{
    TreeViewItemTrait,
};

impl MapTileFixer for Room {
    fn get_tile_planes_mut(&mut self) -> Vec<&mut [u8]> {
        vec![ &mut self.parallax.tiles ]
    }
}

#[derive(Clone, Copy)]
pub struct RoomSize {
    pub width: u32,
    pub height: u32,
}

impl RoomSize {
    pub const ZERO: Self = RoomSize { width: 0, height: 0 };

    pub fn new(width: u32, height: u32) -> Self {
        RoomSize {
            width,
            height,
        }
    }

    pub fn from_room(room: &Room, maps: &AssetList<MapData>) -> Self {
        room.maps.iter().fold(Self::ZERO, |max, room_map| {
            match maps.get(&room_map.map_id) {
                Some(map) => {
                    RoomSize::new(
                        max.width.max(room_map.x as u32 + map.width),
                        max.height.max(room_map.y as u32 + map.height),
                    )
                }
                None => { max }
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RoomEnemyTypeSel {
    Walker,
    Chiller,
    Hopper,
    Floater,
    Other,
}

impl RoomEnemyTypeSel {
    pub fn text(&self) -> &'static str {
        match self {
            RoomEnemyTypeSel::Walker  => { "walker" }
            RoomEnemyTypeSel::Chiller => { "chiller" }
            RoomEnemyTypeSel::Hopper  => { "hopper" }
            RoomEnemyTypeSel::Floater => { "floater" }
            RoomEnemyTypeSel::Other   => { "other..." }
        }
    }
}

impl From<RoomEnemyType> for RoomEnemyTypeSel {
    fn from(value: RoomEnemyType) -> Self {
        match value {
            RoomEnemyType::Walker   => { RoomEnemyTypeSel::Walker }
            RoomEnemyType::Chiller  => { RoomEnemyTypeSel::Chiller }
            RoomEnemyType::Hopper   => { RoomEnemyTypeSel::Hopper }
            RoomEnemyType::Floater  => { RoomEnemyTypeSel::Floater }
            RoomEnemyType::Other(_) => { RoomEnemyTypeSel::Other }
        }
    }
}

impl From<RoomEnemyTypeSel> for RoomEnemyType {
    fn from(value: RoomEnemyTypeSel) -> Self {
        match value {
            RoomEnemyTypeSel::Walker   => { RoomEnemyType::Walker }
            RoomEnemyTypeSel::Chiller  => { RoomEnemyType::Chiller }
            RoomEnemyTypeSel::Hopper   => { RoomEnemyType::Hopper }
            RoomEnemyTypeSel::Floater  => { RoomEnemyType::Floater }
            RoomEnemyTypeSel::Other    => { RoomEnemyType::Other(4) }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoomTriggerTypeSel {
    Unknown,
    Door,
    PlayerSpawn,
    EnemySpawn,
    WallButton,
    DetectorButton,
    UnblockTiles,
    DisableAnimation,
    ChangeTiles,
    RemoveEnemy,
    GetUpgrade,
    GetCollectable,
    GetPickup,
}

impl RoomTriggerTypeSel {
    pub fn from_trigger_type(trigger_type: &RoomTriggerType) -> Self {
        match trigger_type {
            RoomTriggerType::Unknown { .. } => RoomTriggerTypeSel::Unknown,
            RoomTriggerType::Door { .. } => RoomTriggerTypeSel::Door,
            RoomTriggerType::PlayerSpawn { .. } => RoomTriggerTypeSel::PlayerSpawn,
            RoomTriggerType::EnemySpawn { .. } => RoomTriggerTypeSel::EnemySpawn,
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeSel::WallButton }
            RoomTriggerType::DetectorButton {..} => { RoomTriggerTypeSel::DetectorButton }
            RoomTriggerType::UnblockTiles {..} => { RoomTriggerTypeSel::UnblockTiles }
            RoomTriggerType::DisableAnimation {..} => { RoomTriggerTypeSel::DisableAnimation }
            RoomTriggerType::ChangeTiles {..} => { RoomTriggerTypeSel::ChangeTiles }
            RoomTriggerType::RemoveEnemy {..} => { RoomTriggerTypeSel::RemoveEnemy }
            RoomTriggerType::GetUpgrade {..} => { RoomTriggerTypeSel::GetUpgrade }
            RoomTriggerType::GetCollectable {..} => { RoomTriggerTypeSel::GetCollectable }
            RoomTriggerType::GetPickup {..} => { RoomTriggerTypeSel::GetPickup }
        }
    }

    pub fn convert_trigger_type(&self, trigger_type: &mut RoomTriggerType, asset_ids: &AssetIdCollection) -> bool {
        match self {
            RoomTriggerTypeSel::Unknown if ! matches!(trigger_type, RoomTriggerType::Unknown {..}) => {
                *trigger_type = RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0, data4: 0, data5: 0 };
                true
            }
            RoomTriggerTypeSel::PlayerSpawn if ! matches!(trigger_type, RoomTriggerType::PlayerSpawn {..}) => {
                *trigger_type = RoomTriggerType::PlayerSpawn { direction: RoomEntityDirection::Right };
                true
            }
            RoomTriggerTypeSel::EnemySpawn if ! matches!(trigger_type, RoomTriggerType::EnemySpawn {..}) => {
                if let Some(animation_id) = asset_ids.animations.get_first() {
                    *trigger_type = RoomTriggerType::EnemySpawn {
                        enemy_id: 0,
                        enemy_type: RoomEnemyType::Walker,
                        animation_id,
                        direction: RoomEntityDirection::Right,
                        req_flag_id: u16::MAX,
                    };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::Door if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                if let Some(room_id) = asset_ids.rooms.get_first() {
                    *trigger_type = RoomTriggerType::Door { dest_room_id: room_id, dest_trigger_id: 0, req_flag_id: u16::MAX };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::WallButton if ! matches!(trigger_type, RoomTriggerType::WallButton {..}) => {
                *trigger_type = RoomTriggerType::WallButton {
                    width: 16,
                    height: 16,
                    set_effect_id: u16::MAX,
                    set_flag_id: u16::MAX,
                    req_flag_id: u16::MAX,
                    req_collectable_id: u16::MAX,
                };
                true
            }
            RoomTriggerTypeSel::DetectorButton if ! matches!(trigger_type, RoomTriggerType::DetectorButton {..}) => {
                *trigger_type = RoomTriggerType::DetectorButton {
                    width: 16,
                    height: 16,
                    set_effect_id: u16::MAX,
                    set_flag_id: u16::MAX,
                    req_flag_id: u16::MAX,
                    req_collectable_id: u16::MAX,
                };
                true
            }
            RoomTriggerTypeSel::UnblockTiles if ! matches!(trigger_type, RoomTriggerType::UnblockTiles {..}) => {
                *trigger_type = RoomTriggerType::UnblockTiles {
                    width: 64,
                    height: 64,
                    on_effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::DisableAnimation if ! matches!(trigger_type, RoomTriggerType::DisableAnimation {..}) => {
                *trigger_type = RoomTriggerType::DisableAnimation {
                    width: 64,
                    height: 64,
                    on_effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::ChangeTiles if ! matches!(trigger_type, RoomTriggerType::ChangeTiles {..}) => {
                *trigger_type = RoomTriggerType::ChangeTiles {
                    map_index: 0,
                    tile_change_index: 0,
                    map_layer: RoomTriggerMapLayer::Background2,
                    on_effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::RemoveEnemy if ! matches!(trigger_type, RoomTriggerType::RemoveEnemy {..}) => {
                *trigger_type = RoomTriggerType::RemoveEnemy {
                    enemy_id: 0,
                    on_effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetUpgrade if ! matches!(trigger_type, RoomTriggerType::GetUpgrade {..}) => {
                *trigger_type = RoomTriggerType::GetUpgrade {
                    get_upgrade_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetCollectable if ! matches!(trigger_type, RoomTriggerType::GetCollectable {..}) => {
                *trigger_type = RoomTriggerType::GetCollectable {
                    get_collectable_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetPickup if ! matches!(trigger_type, RoomTriggerType::GetPickup {..}) => {
                *trigger_type = RoomTriggerType::GetPickup {
                    get_pickup_id: u16::MAX,
                    respawn_delay: 0,
                };
                true
            }
            _ => { false }
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            RoomTriggerTypeSel::Unknown => "any",
            RoomTriggerTypeSel::Door => "door",
            RoomTriggerTypeSel::PlayerSpawn => "player spawn",
            RoomTriggerTypeSel::EnemySpawn => "enemy spawn",
            RoomTriggerTypeSel::WallButton => { "wall button" }
            RoomTriggerTypeSel::DetectorButton => { "detector button" }
            RoomTriggerTypeSel::UnblockTiles => { "unblock tiles" }
            RoomTriggerTypeSel::DisableAnimation => { "disable animation" }
            RoomTriggerTypeSel::ChangeTiles => { "change tiles" }
            RoomTriggerTypeSel::RemoveEnemy => { "remove enemy" }
            RoomTriggerTypeSel::GetUpgrade => { "upgrade" }
            RoomTriggerTypeSel::GetCollectable => { "collectable" }
            RoomTriggerTypeSel::GetPickup => { "pickup" }
        }
    }
}

impl TreeViewItemTrait for RoomTrigger {
    fn name(&self) -> &str {
        &self.name_id
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_trigger_type = RoomTriggerTypeSel::from_trigger_type(&self.trigger_type);
        let other_trigger_type = RoomTriggerTypeSel::from_trigger_type(&other.trigger_type);
        let cmp = self_trigger_type.cmp(&other_trigger_type);
        if cmp == std::cmp::Ordering::Equal {
            Some(self.name_id.cmp(&other.name_id))
        } else {
            Some(cmp)
        }
    }
}

pub struct RoomParallaxUndoData {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u8>,
}

impl RoomParallaxUndoData {
    pub fn from_parallax(para: &RoomParallax) -> Self {
        let num_tiles = (para.width * para.height) as usize;
        let mut tiles = Vec::with_capacity(num_tiles);
        for y in 0..para.height {
            for x in 0..para.width {
                let tile_index = (para.width * y + x) as usize;
                tiles.push(para.tiles[tile_index]);
            }
        }
        RoomParallaxUndoData {
            width: para.width,
            height: para.height,
            tiles,
        }
    }

    pub fn to_parallax(&self, para: &mut RoomParallax) -> bool {
        if self.width != para.width || self.height != para.height  {
            return false;
        }
        let width = para.width as usize;
        for y in 0..para.height as usize {
            let tile_index = width * y;
            para.tiles[tile_index..tile_index+width].copy_from_slice(&self.tiles[tile_index..tile_index+width]);
        }
        true
    }
}

#[derive(Clone)]
pub struct DrawRoomLayerInfo {
    pub zoom: f32,
    pub pos: egui::Pos2,
    pub screen_rect: egui::Rect,
}

pub struct DrawRoomLayerData<'a> {
    pub tileset: &'a Tileset,
    pub slot: TextureSlot,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub tiles: &'a [u8],
}

pub fn draw_room_layer<'a>(
    ui: &mut egui::Ui,
    wc: &mut WindowContext,
    info: &DrawRoomLayerInfo,
    layer: &DrawRoomLayerData<'a>,
    layer_tint: Option<egui::Color32>
) {
    for y in 0..layer.height {
        for x in 0..layer.width {
            let tile = layer.tiles[(y * layer.stride + x) as usize];
            if tile == MapData::NO_TILE { continue; }
            let (uv, texture) = if tile as u32 >= layer.tileset.num_tiles {
                (FULL_UV, STATIC_IMAGES.bad_tile().texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent))
            } else {
                (layer.tileset.get_item_uv(tile as u32), layer.tileset.texture(wc.tex_man, wc.egui.ctx, layer.slot))
            };
            let tile_rect = egui::Rect::from_min_max(
                info.pos + info.zoom * TILE_SIZE * egui::Vec2::new(x as f32, y as f32),
                info.pos + info.zoom * TILE_SIZE * egui::Vec2::new((x+1) as f32, (y+1) as f32)
            );
            if tile_rect.intersects(info.screen_rect) {
                let image = egui::Image::from_texture((texture.id(), egui::Vec2::splat(TILE_SIZE))).uv(uv);
                match layer_tint {
                    Some(tint) => { image.tint(tint).paint_at(ui, tile_rect) }
                    None => { image.paint_at(ui, tile_rect) }
                }
            }
        }
    }
}
