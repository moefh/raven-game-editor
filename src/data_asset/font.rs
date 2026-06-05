#[derive(std::hash::Hash)]
pub struct Font {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Font {
    pub const FIRST_CHAR: u32 = 32;
    pub const NUM_CHARS: u32 = 96;
    pub const BG_COLOR: u8 = 0b00_111_000;
    pub const FG_COLOR: u8 = 0b11_000_000;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let width = 6;
        let height = 8;
        Font {
            asset: super::DataAsset::new(super::DataAssetType::Font, id, name),
            width,
            height,
            data: vec![Self::BG_COLOR; (width * height * Font::NUM_CHARS) as usize],
        }
    }
}

impl super::DuplicableAsset<Font> for Font {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        Font {
            asset: self.asset.duplicate(dup_id, dup_name),
            width: self.width,
            height: self.height,
            data: self.data.clone(),
        }
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
