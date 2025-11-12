mod tileset;
mod map_data;
mod room;
mod sprite;
mod sprite_animation;
mod sfx;
mod mod_data;
mod font;
mod prop_font;
mod widgets;

pub use tileset::TilesetEditor;
pub use map_data::MapDataEditor;
pub use room::{RoomEditor, RoomEditorAssetLists};
pub use sprite::SpriteEditor;
pub use sprite_animation::SpriteAnimationEditor;
pub use sfx::SfxEditor;
pub use mod_data::ModDataEditor;
pub use font::FontEditor;
pub use prop_font::PropFontEditor;

use crate::data_asset::{DataAssetId, ImageCollectionAsset};

pub struct DataAssetEditor {
    pub id: DataAssetId,
    pub open: bool,
}

impl DataAssetEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        DataAssetEditor {
            id,
            open,
        }
    }
}

pub fn raven_color_to_rgb(raven_color: u8) -> egui::Color32 {
    let r = raven_color & 0x03;
    let g = (raven_color & 0x0c) >> 2;
    let b = (raven_color & 0x30) >> 4;
    let cr = (r << 6) | (r << 4) | (r << 2) | r;
    let cg = (g << 6) | (g << 4) | (g << 2) | g;
    let cb = (b << 6) | (b << 4) | (b << 2) | b;
    egui::Color32::from_rgb(cr, cg, cb)
}

pub fn calc_image_editor_window_size(image: &impl ImageCollectionAsset) -> (egui::Vec2, egui::Vec2) {
    let img_w = image.width() as f32;
    let img_h = image.height() as f32;
    let min_width = 130.0 + img_w + 220.0;
    let min_height = 2.0 * img_h + 80.0;
    let min_size = egui::Vec2::new(min_width, min_height);
    let default_size = egui::Vec2::new(min_width + 5.0 * img_w, min_height + 200.0).max(egui::Vec2::new(min_width + 200.0, 0.0));
    (min_size, default_size)
}

pub fn create_editor_window<'a>(id: DataAssetId, title: &'a str, wc: &crate::misc::WindowContext) -> egui::Window<'a> {
    let editor_id = egui::Id::new(format!("editor_{}", id));
    let default_rect = egui::Rect {
        min: egui::Pos2 {
            x : wc.window_space.min.x + 10.0,
            y : wc.window_space.min.y + 10.0,
        },
        max: egui::Pos2 {
            x: 600.0,
            y: 300.0,
        }
    };
    let frame = egui::Frame::window(&wc.egui.ctx.style()).inner_margin(1.0);
    egui::Window::new(title)
        .id(editor_id)
        .frame(frame)
        .default_rect(default_rect)
        .max_size(wc.window_space.size())
        .constrain_to(wc.window_space)
}
