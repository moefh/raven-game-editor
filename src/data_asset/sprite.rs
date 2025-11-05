pub struct Sprite {
    pub asset: super::DataAsset,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub num_frames: u32,
    pub data: Vec<u32>,
}

impl Sprite {

    pub fn new(asset: super::DataAsset) -> Self {
        Sprite {
            asset,
            width: 0,
            height: 0,
            stride: 0,
            num_frames: 0,
            data: Vec::new(),
        }
    }
    
}
