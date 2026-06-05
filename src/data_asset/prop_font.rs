#[derive(std::hash::Hash)]
pub struct PropFont {
    pub asset: super::DataAsset,
    pub max_width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub char_widths: Vec<u8>,
}

impl PropFont {
    pub const FIRST_CHAR: u32 = 32;
    pub const NUM_CHARS: u32 = 96;
    pub const BG_COLOR: u8 = 0b00_111_000;
    pub const FG_COLOR: u8 = 0b11_000_000;

    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let height = 8;
        let max_width = 2 * height;
        PropFont {
            asset: super::DataAsset::new(super::DataAssetType::PropFont, id, name),
            max_width,
            height,
            data: vec![Self::BG_COLOR; (max_width * height * PropFont::NUM_CHARS) as usize],
            char_widths: vec![6; PropFont::NUM_CHARS as usize],
        }
    }
}

impl super::DuplicableAsset<PropFont> for PropFont {
    fn duplicate(&self, dup_id: super::DataAssetId, dup_name: String) -> Self {
        PropFont {
            asset: self.asset.duplicate(dup_id, dup_name),
            max_width: self.max_width,
            height: self.height,
            data: self.data.clone(),
            char_widths: self.char_widths.clone(),
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
