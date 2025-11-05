pub struct Font {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Font {

    pub fn new(asset: super::DataAsset) -> Self {
        Font {
            asset,
            width: 0,
            height: 0,
            data: Vec::new(),
        }
    }
    
}
