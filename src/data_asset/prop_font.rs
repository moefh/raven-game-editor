#[derive(std::hash::Hash)]
pub struct PropFont {
    pub asset: super::DataAsset,
    pub max_width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub char_widths: Vec<u8>,
}

pub struct CreationData<'a> {
    pub height: u32,
    pub data: &'a [u8],
    pub char_widths: Vec<u8>,
    pub char_offsets: Vec<u16>,
}

impl PropFont {
    pub const FIRST_CHAR: u32 = 32;
    pub const NUM_CHARS: u32 = 96;
    pub const BG_COLOR: u8 = 0b001100;
    pub const FG_COLOR: u8 = 0b110000;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let height = 8;
        let max_width = 2 * height;
        PropFont {
            asset: super::DataAsset::new(super::DataAssetType::PropFont, id, name),
            max_width,
            height,
            data: vec![0x0c; (max_width * height * PropFont::NUM_CHARS) as usize],
            char_widths: vec![6; PropFont::NUM_CHARS as usize],
        }
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        PropFont {
            asset: super::DataAsset::new(super::DataAssetType::PropFont, id, name),
            max_width: 2 * data.height,
            height: data.height,
            data: Self::bits_to_pixels(data.data, &data.char_widths, data.height, &data.char_offsets),
            char_widths: data.char_widths,
        }
    }

    fn bits_to_pixels(bits: &[u8], widths: &[u8], height: u32, offsets: &[u16]) -> Vec<u8> {
        let height = height as usize;
        let num_items = Self::NUM_CHARS as usize;
        let max_width = 2 * height;
        let mut pixels = vec![Self::BG_COLOR; max_width * height * num_items];
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
                            pixels[((ch * height) + y) * max_width + x*8 + ix] = Self::FG_COLOR;
                        }
                    }
                }
            }
        }
        pixels
    }
}

impl super::GenericAsset for PropFont {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: height(1) + pad(3) + data<ptr>(4) + 96*char_width(1) + 96*char_offset(2)
        let header = 4usize + 4usize + 96usize + 96usize * 2usize;

        // data
        let data = self.char_widths.iter().fold(0, |acc, v| { acc + self.height as usize * v.div_ceil(8) as usize });

        header + data
    }
}
