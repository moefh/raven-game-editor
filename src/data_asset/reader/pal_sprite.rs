use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    PalSprite,
    PalSpriteDepth,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
    pub bits_per_pixel: u32,
    pub palette: Vec<u8>,
    pub pixels: Vec<u8>,
}

impl CreationData {
    pub fn into_pal_sprite(self) -> PalSprite {
        let color_to_palette_index_map = PalSprite::gen_color_to_palette_index_map(&self.palette);
        PalSprite {
            asset: DataAsset::new(DataAssetType::PalSprite, self.asset_id, self.name),
            width: self.width,
            height: self.height,
            num_frames: self.num_frames,
            depth: PalSpriteDepth::from_bits_per_pixel(self.bits_per_pixel),
            palette: self.palette,
            color_to_palette_index_map,
            data: self.pixels,
        }
    }
}
