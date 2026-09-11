use std::collections::{BTreeMap, HashSet};

use crate::data_asset::{DataAssetId, DataAssetStore, PalSprite};

use super::{
    AssetError,
    AssetWarning,
};

fn check_pal_sprite(pal_sprite: &PalSprite) -> (Vec<AssetError>, Vec<AssetWarning>) {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    // check number of frames
    if pal_sprite.num_frames > 255 {
        errors.push(AssetError::PalSpriteTooBig { num_frames: pal_sprite.num_frames });
    }

    // check that every pixel has a palette color
    let pal_colors: HashSet<u8> = HashSet::from_iter(pal_sprite.palette.iter().copied());
    let frame_len = (pal_sprite.width * pal_sprite.height) as usize;
    let mut num_pixels = 0u64;
    let mut first_bad_frame = None;
    for (num, color) in pal_sprite.data.iter().enumerate() {
        if ! pal_colors.contains(color) {
            if first_bad_frame.is_none() {
                first_bad_frame = Some((num / frame_len) as u32);
            }
            num_pixels = num_pixels.saturating_add(1);
        }
    }
    if let Some(frame_num) = first_bad_frame {
        errors.push(AssetError::PalSpriteColorOutOfPalette { frame_num, num_pixels });
    }

    (errors, warnings)
}

pub fn check_pal_sprites(
    asset_errors: &mut BTreeMap<DataAssetId, Vec<AssetError>>,
    asset_warnings: &mut BTreeMap<DataAssetId, Vec<AssetWarning>>,
    store: &DataAssetStore
) {
    for pal_sprite in store.assets.pal_sprites.iter() {
        let (errors, warnings) = check_pal_sprite(pal_sprite);
        asset_errors.insert(pal_sprite.asset.id, errors);
        asset_warnings.insert(pal_sprite.asset.id, warnings);
    }
}
