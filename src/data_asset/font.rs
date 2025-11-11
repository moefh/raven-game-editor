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

    pub fn new(asset: super::DataAsset) -> Self {
        let width = 6;
        let height = 8;
        Font {
            asset,
            width,
            height,
            data: vec![0x0c; (width * height * Font::NUM_CHARS) as usize],
        }
    }

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Font {
            asset,
            width: data.width,
            height: data.height,
            data: super::image_pixels_font_to_u8(data.data, data.width, data.height, Font::NUM_CHARS),
        }
    }
}

impl super::GenericAsset for Font {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: width(1) + height(1) + pad(2) + data<ptr>(4)
        let header = 4usize + 4usize;

        // data: 96 * ceil(width/8) * height
        let data = (Font::NUM_CHARS * self.width.div_ceil(8) * self.height) as usize;

        header + data
    }
}

impl super::ImageCollectionAsset for Font {
    fn asset_id(&self) -> super::DataAssetId { self.asset.id }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn num_items(&self) -> u32 { Font::NUM_CHARS }
    fn data(&self) -> &[u8] { &self.data }
}
