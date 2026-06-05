use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    Font,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl CreationData {
    pub fn into_font(self) -> Font {
        Font {
            asset: DataAsset::new(DataAssetType::Font, self.asset_id, self.name),
            width: self.width,
            height: self.height,
            data: Self::bits_to_pixels(&self.data, self.width, self.height),
        }
    }

    pub fn bits_to_pixels(bits: &[u8], width: u32, height: u32) -> Vec<u8> {
        let stride = width.div_ceil(8) as usize;
        let mut pixels = vec![Font::BG_COLOR; (width * height * Font::NUM_CHARS) as usize];
        for y in 0..(height * Font::NUM_CHARS) as usize {
            for x in 0..stride {
                let block = bits.get(y*stride + x).map_or(0, |&v| v);
                for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                    if block & (1 << ix) != 0 {
                        pixels[y * width as usize + x*8 + ix] = Font::FG_COLOR;
                    }
                }
            }
        }
        pixels
    }
}
