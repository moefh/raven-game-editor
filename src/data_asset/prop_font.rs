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

    pub fn new(asset: super::DataAsset) -> Self {
        let height = 8;
        let max_width = 2 * height;
        PropFont {
            asset,
            max_width,
            height,
            data: vec![0x0c; (max_width * height * PropFont::NUM_CHARS) as usize],
            char_widths: vec![6; PropFont::NUM_CHARS as usize],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        PropFont {
            asset,
            max_width: 2 * data.height,
            height: data.height,
            data: super::image_pixels_prop_font_to_u8(data.data, &data.char_widths, data.height,
                                                      PropFont::NUM_CHARS, &data.char_offsets),
            char_widths: data.char_widths,
        }
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
