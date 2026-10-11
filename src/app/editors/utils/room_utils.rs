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
    RoomTriggerType,
    RoomEntityDirection,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoomTriggerTypeSel {
    Unknown,
    Door,
    Trap,
    PlayerSpawn,
    EnemySpawn,
    WallButton,
    FloorButton,
    UnblockEffect,
    DisableAnimationEffect,
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
            RoomTriggerType::Trap { .. } => RoomTriggerTypeSel::Trap,
            RoomTriggerType::WallButton {..} => { RoomTriggerTypeSel::WallButton }
            RoomTriggerType::FloorButton {..} => { RoomTriggerTypeSel::FloorButton }
            RoomTriggerType::UnblockEffect {..} => { RoomTriggerTypeSel::UnblockEffect }
            RoomTriggerType::DisableAnimationEffect {..} => { RoomTriggerTypeSel::DisableAnimationEffect }
            RoomTriggerType::GetUpgrade {..} => { RoomTriggerTypeSel::GetUpgrade }
            RoomTriggerType::GetCollectable {..} => { RoomTriggerTypeSel::GetCollectable }
            RoomTriggerType::GetPickup {..} => { RoomTriggerTypeSel::GetPickup }
        }
    }

    pub fn convert_trigger_type(&self, trigger_type: &mut RoomTriggerType, asset_ids: &AssetIdCollection) -> bool {
        match self {
            RoomTriggerTypeSel::Unknown if ! matches!(trigger_type, RoomTriggerType::Unknown {..}) => {
                *trigger_type = RoomTriggerType::Unknown { data0: 0, data1: 0, data2: 0, data3: 0 };
                true
            }
            RoomTriggerTypeSel::Trap if ! matches!(trigger_type, RoomTriggerType::Trap {..}) => {
                *trigger_type = RoomTriggerType::Trap { width: 64, height: 64, trap_type: 0 };
                true
            }
            RoomTriggerTypeSel::PlayerSpawn if ! matches!(trigger_type, RoomTriggerType::PlayerSpawn {..}) => {
                *trigger_type = RoomTriggerType::PlayerSpawn { direction: RoomEntityDirection::Right };
                true
            }
            RoomTriggerTypeSel::EnemySpawn if ! matches!(trigger_type, RoomTriggerType::EnemySpawn {..}) => {
                if let Some(animation_id) = asset_ids.animations.get_first() {
                    *trigger_type = RoomTriggerType::EnemySpawn {
                        animation_id,
                        enemy_type: RoomEnemyType::Walker,
                        direction: RoomEntityDirection::Right,
                    };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::Door if ! matches!(trigger_type, RoomTriggerType::Door {..}) => {
                if let Some(room_id) = asset_ids.rooms.get_first() {
                    *trigger_type = RoomTriggerType::Door { dest_room_id: room_id, dest_trigger_id: 0 };
                    true
                } else {
                    false
                }
            }
            RoomTriggerTypeSel::WallButton if ! matches!(trigger_type, RoomTriggerType::WallButton {..}) => {
                *trigger_type = RoomTriggerType::WallButton {
                    width: 16,
                    height: 16,
                    effect_id: u16::MAX,
                    flag_id: u16::MAX,
                    required_collectable_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::FloorButton if ! matches!(trigger_type, RoomTriggerType::FloorButton {..}) => {
                *trigger_type = RoomTriggerType::FloorButton {
                    width: 16,
                    height: 16,
                    effect_id: u16::MAX,
                    flag_id: u16::MAX,
                };
                true
            }
            RoomTriggerTypeSel::UnblockEffect if ! matches!(trigger_type, RoomTriggerType::UnblockEffect {..}) => {
                *trigger_type = RoomTriggerType::UnblockEffect {
                    width: 64,
                    height: 64,
                    effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::DisableAnimationEffect if ! matches!(trigger_type, RoomTriggerType::DisableAnimationEffect {..}) => {
                *trigger_type = RoomTriggerType::DisableAnimationEffect {
                    width: 64,
                    height: 64,
                    effect_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetUpgrade if ! matches!(trigger_type, RoomTriggerType::GetUpgrade {..}) => {
                *trigger_type = RoomTriggerType::GetUpgrade {
                    upgrade_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetCollectable if ! matches!(trigger_type, RoomTriggerType::GetCollectable {..}) => {
                *trigger_type = RoomTriggerType::GetCollectable {
                    collectable_id: u16::MAX
                };
                true
            }
            RoomTriggerTypeSel::GetPickup if ! matches!(trigger_type, RoomTriggerType::GetPickup {..}) => {
                *trigger_type = RoomTriggerType::GetPickup {
                    pickup_id: u16::MAX
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
            RoomTriggerTypeSel::Trap => "trap",
            RoomTriggerTypeSel::PlayerSpawn => "player spawn",
            RoomTriggerTypeSel::EnemySpawn => "enemy spawn",
            RoomTriggerTypeSel::WallButton => { "wall button" }
            RoomTriggerTypeSel::FloorButton => { "floor button" }
            RoomTriggerTypeSel::UnblockEffect => { "unblock" }
            RoomTriggerTypeSel::DisableAnimationEffect => { "disable animation" }
            RoomTriggerTypeSel::GetUpgrade => { "upgrade" }
            RoomTriggerTypeSel::GetCollectable => { "collectable" }
            RoomTriggerTypeSel::GetPickup => { "pickup" }
        }
    }
}

#[derive(Clone, Copy)]
pub struct RoomParallaxRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl RoomParallaxRect {
    pub fn from_rect(rect: egui::Rect, para: &RoomParallax) -> Option<Self> {
        let rect = rect.intersect(egui::Rect::from_min_max(
            egui::Pos2::ZERO,
            egui::Pos2::new(para.width as f32, para.height as f32)
        ));
        Some(RoomParallaxRect {
            x: rect.min.x as u32,
            y: rect.min.y as u32,
            width: rect.width().max(0.0) as u32,
            height: rect.height().max(0.0) as u32,
        })
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
