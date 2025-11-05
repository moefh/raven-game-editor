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
