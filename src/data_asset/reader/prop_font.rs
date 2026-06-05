use super::super::{
    DataAsset,
    DataAssetId,
    DataAssetType,
    PropFont,
};

pub struct CreationData {
    pub asset_id: DataAssetId,
    pub name: String,
    pub height: u32,
    pub data: Vec<u8>,
    pub char_widths: Vec<u8>,
    pub char_offsets: Vec<u16>,
}

impl CreationData {
    pub fn into_prop_font(self) -> PropFont {
        PropFont {
            asset: DataAsset::new(DataAssetType::PropFont, self.asset_id, self.name),
            max_width: 2 * self.height,
            height: self.height,
            data: Self::bits_to_pixels(&self.data, &self.char_widths, self.height, &self.char_offsets),
            char_widths: self.char_widths,
        }
    }

    fn bits_to_pixels(bits: &[u8], widths: &[u8], height: u32, offsets: &[u16]) -> Vec<u8> {
        let height = height as usize;
        let num_items = PropFont::NUM_CHARS as usize;
        let max_width = 2 * height;
        let mut pixels = vec![PropFont::BG_COLOR; max_width * height * num_items];
        if widths.len() != num_items || offsets.len() != widths.len() { return pixels; }
        for ch in 0..num_items {
            let offset = offsets[ch] as usize;
            let width = widths[ch] as usize;
            let stride = width.div_ceil(8);
            for y in 0..height {
                for x in 0..stride {
                    let block = bits.get(offset + y*stride + x).map_or(0, |&v| v);
                    for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                        if block & (1 << ix) != 0 && (x*8 + ix) < max_width {
                            pixels[((ch * height) + y) * max_width + x*8 + ix] = PropFont::FG_COLOR;
                        }
                    }
                }
            }
        }
        pixels
    }
}
