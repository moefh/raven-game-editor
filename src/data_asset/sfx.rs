#[allow(unused)]
pub struct Sfx {
    pub asset: super::DataAsset,
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub bits_per_sample: u16,
    pub samples: Vec<i16>,
}

impl Sfx {

    pub fn new(asset: super::DataAsset) -> Self {
        Sfx {
            asset,
            len: 0,
            loop_start: 0,
            loop_len: 0,
            bits_per_sample: 8,
            samples: Vec::new(),
        }
    }

}
