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

impl super::GenericAsset for Font {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: width(1) + height(1) + pad(2) + data<ptr>(4)
        let header = 4usize + 4usize;

        // data: 96 * ceil(width/8) * height
        let data = 96usize * (self.width.div_ceil(8) as usize) * (self.height as usize);

        header + data
    }
}
