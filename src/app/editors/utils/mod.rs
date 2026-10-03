mod map_utils;
mod room_utils;
mod image_utils;
mod sprite_utils;
mod tileset_utils;
mod asset_id_holder;
mod pal_sprite_utils;
mod image_zoom_option;
mod data_table_utils;
pub mod world_grid;

pub use map_utils::{*};
pub use room_utils::{*};
pub use image_utils::{*};
pub use sprite_utils::{*};
pub use tileset_utils::{*};
pub use asset_id_holder::{*};
pub use pal_sprite_utils::{*};
pub use data_table_utils::{*};
pub use image_zoom_option::{*};

use crate::platform::current_time_as_millis;

use super::WindowContext;

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

pub fn get_animation_step(wc: &WindowContext) -> u32 {
    const ANIMATION_LOOP_MOD: u64 = 3628800;

    ((current_time_as_millis() / wc.settings.animation_ms_per_frame as u64) % ANIMATION_LOOP_MOD) as u32
}

pub fn get_game_runner_step(wc: &WindowContext) -> u32 {
    const LOOP_MOD: u64 = 3628800;

    ((current_time_as_millis() / wc.settings.game_runner_ms_per_frame as u64) % LOOP_MOD) as u32
}
