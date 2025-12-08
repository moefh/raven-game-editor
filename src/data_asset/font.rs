pub struct Font {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct CreationData<'a> {
    pub width: u32,
    pub height: u32,
    pub data: &'a [u8],
}

impl Font {
    pub const FIRST_CHAR: u32 = 32;
    pub const NUM_CHARS: u32 = 96;
    pub const BG_COLOR: u8 = 0b001100;
    pub const FG_COLOR: u8 = 0b110000;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 6;
        let height = 8;
        Font {
            asset: super::DataAsset::new(super::DataAssetType::Font, id, name),
            width,
            height,
            data: vec![0x0c; (width * height * Font::NUM_CHARS) as usize],
        }
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        Font {
            asset: super::DataAsset::new(super::DataAssetType::Font, id, name),
            width: data.width,
            height: data.height,
            data: Self::bits_to_pixels(data.data, data.width, data.height),
        }
    }

    fn bits_to_pixels(bits: &[u8], width: u32, height: u32) -> Vec<u8> {
        let stride = width.div_ceil(8) as usize;
        let mut pixels = vec![Font::BG_COLOR; (width * height * Self::NUM_CHARS) as usize];
        for y in 0..(height * Self::NUM_CHARS) as usize {
            for x in 0..stride {
                let block = bits.get(y*stride + x).map_or(0, |&v| v);
                for ix in 0..8.min(width as i32 - x as i32 * 8) as usize {
                    if block & (1 << ix) != 0 {
                        pixels[y * width as usize + x*8 + ix] = Self::FG_COLOR;
                    }
                }
            }
        }
        pixels
    }
}

impl super::GenericAsset for Font {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: width(1) + height(1) + pad(2) + data<ptr>(4)
        let header = 4usize + 4usize;

        // data: 96 * ceil(width/8) * height
        let data = (Font::NUM_CHARS * self.width.div_ceil(8) * self.height) as usize;

        header + data
    }
}
