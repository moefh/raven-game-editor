pub mod asset_defs;
pub mod image_table;
pub mod mod_utils;
pub mod wav_utils;
pub mod reader;
pub mod writer;

pub use image_table::{IMAGES, IMAGE_REFS, STATIC_IMAGES};

pub fn current_time_as_millis() -> u64 {
    use std::time::{SystemTime, Duration, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    since_the_epoch.as_millis() as u64
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
