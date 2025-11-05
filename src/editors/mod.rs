mod tileset;
mod map_data;
mod room;
mod sprite;
mod sprite_animation;
mod sfx;
mod mod_data;
mod font;
mod prop_font;

pub use tileset::TilesetEditor;
pub use map_data::MapDataEditor;
pub use room::RoomEditor;
pub use sprite::SpriteEditor;
pub use sprite_animation::SpriteAnimationEditor;
pub use sfx::SfxEditor;
pub use mod_data::ModDataEditor;
pub use font::FontEditor;
pub use prop_font::PropFontEditor;

use crate::data_asset::DataAssetId;

pub struct DataAssetEditor {
    pub id: DataAssetId,
    pub open: bool,
}

pub fn create_editor_window<'a>(id: DataAssetId, title: &'a str, window_space: egui::Rect) -> egui::Window<'a> {
    let editor_id = egui::Id::new(format!("editor_{}", id));
    let default_rect = egui::Rect {
        min: egui::Pos2 {
            x : window_space.min.x + 10.0,
            y : window_space.min.y + 10.0,
        },
        max: egui::Pos2 {
            x: 600.0,
            y: 300.0,
        }
    };
    egui::Window::new(title)
        .id(editor_id)
        .default_rect(default_rect)
        .max_width(window_space.max.x - window_space.min.x)
        .max_height(window_space.max.y - window_space.min.y)
        .constrain_to(window_space)
}
