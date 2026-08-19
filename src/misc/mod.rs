pub mod asset_defs;
pub mod image_table;
pub mod mod_utils;
pub mod wav_utils;
pub mod reader;
pub mod writer;

pub use image_table::{IMAGES, IMAGE_REFS, STATIC_IMAGES, get_asset_type_image};

pub fn calc_hash(h: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;

    let mut hasher = std::hash::DefaultHasher::new();
    h.hash(&mut hasher);
    hasher.finish()
}
