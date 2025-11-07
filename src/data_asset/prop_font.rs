pub struct PropFont {
    pub asset: super::DataAsset,
    pub height: u32,
    pub data: Vec<u8>,
    pub char_widths: Vec<u8>,
    pub char_offsets: Vec<u16>,
}

impl PropFont {
    pub fn new(asset: super::DataAsset) -> Self {
        PropFont {
            asset,
            height: 0,
            data: Vec::new(),
            char_widths: Vec::new(),
            char_offsets: Vec::new(),
        }
    }
}

impl super::GenericAsset for PropFont {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: height(1) + pad(3) + data<ptr>(4) + 96*char_width(1) + 96*char_offset(2)
        let header = 4usize + 4usize + 96usize + 96usize * 2usize;

        // data: TODO
        let data = self.data.len();

        header + data
    }
}
