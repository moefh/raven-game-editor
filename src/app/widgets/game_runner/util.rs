use crate::data_asset::{
    DataAssetStore,
    Room,
    RoomTrigger,
    RoomTriggerType,
    Tileset,
    SpriteAnimation,
};

use super::super::super::editors::{
    RoomSize,
};

pub const TILE_SIZE: f32 = Tileset::TILE_SIZE as f32;

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
