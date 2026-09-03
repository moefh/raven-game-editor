use crate::data_asset::{
    DataAssetStore,
    Room,
    RoomTrigger,
    RoomTriggerType,
    RoomEntityDirection,
    Tileset,
    SpriteAnimation,
};

use super::super::super::editors::{
    RoomSize,
};

pub const TILE_SIZE: f32 = Tileset::TILE_SIZE as f32;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Right,
    Left,
}

impl Direction {
    pub fn dx(self) -> i32 {
        match self {
            Direction::Right => { 1 }
            Direction::Left => { -1 }
        }
    }

    pub fn flip(self) -> Self {
        match self {
            Direction::Right => { Direction::Left }
            Direction::Left => { Direction::Right }
        }
    }
}

impl From<u8> for Direction {
    fn from(val: u8) -> Self {
        if val != 0 {
            Direction::Left
        } else {
            Direction::Right
        }
    }
}

impl From<RoomEntityDirection> for Direction {
    fn from(val: RoomEntityDirection) -> Self {
        match val {
            RoomEntityDirection::Left => { Direction::Left }
            RoomEntityDirection::Right => { Direction::Right }
        }
    }
}

pub fn get_sprite_animation_by_name<'a>(store: &'a DataAssetStore, name: &str) -> Option<&'a SpriteAnimation> {
    store.assets.animations.iter().find(|asset| asset.asset.name == name)
}

pub fn get_room_player_spawn(room: &Room) -> Option<&RoomTrigger> {
    room.triggers.iter().find(|tr| matches!(tr.trigger_type, RoomTriggerType::PlayerSpawn {..}))
}

pub fn get_room_size(room: &Room, store: &DataAssetStore) -> RoomSize {
    let room_size_in_tiles = RoomSize::from_room(room, &store.assets.maps);
    RoomSize::new(room_size_in_tiles.width * Tileset::TILE_SIZE, room_size_in_tiles.height * Tileset::TILE_SIZE)
}
