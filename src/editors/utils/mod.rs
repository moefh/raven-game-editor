mod map_utils;
mod room_utils;
mod image_utils;
mod tileset_utils;
mod asset_id_holder;
pub mod world_grid;

pub use map_utils::{*};
pub use room_utils::{*};
pub use image_utils::{*};
pub use tileset_utils::{*};
pub use asset_id_holder::{*};

use crate::data_asset::{
    Room,
    MapData,
    AssetList,
};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum RectBorder {
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
}

impl RectBorder {
    pub fn cursor(&self) -> egui::CursorIcon {
        match self {
            RectBorder::Left | RectBorder::Right => egui::CursorIcon::ResizeHorizontal,
            RectBorder::Top | RectBorder::Bottom => egui::CursorIcon::ResizeVertical,
            RectBorder::TopLeft | RectBorder::BottomRight => egui::CursorIcon::ResizeNwSe,
            RectBorder::TopRight | RectBorder::BottomLeft => egui::CursorIcon::ResizeNeSw,
        }
    }
}

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
