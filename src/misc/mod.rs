pub mod asset_defs;
pub mod image_table;
pub mod mod_utils;
pub mod wav_utils;
pub mod reader;
pub mod writer;

pub use image_table::{IMAGES, IMAGE_REFS, STATIC_IMAGES, ImageRef, get_asset_type_image};

pub fn calc_hash(h: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;

    let mut hasher = std::hash::DefaultHasher::new();
    h.hash(&mut hasher);
    hasher.finish()
}

pub fn current_time_as_millis() -> u64 {
    use std::time::{SystemTime, Duration, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    since_the_epoch.as_millis() as u64
}
