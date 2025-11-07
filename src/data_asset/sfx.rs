#[allow(unused)]
pub struct Sfx {
    pub asset: super::DataAsset,
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub bits_per_sample: u16,
    pub samples: Vec<i16>,
}

pub struct CreationData<'a> {
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub bits_per_sample: u16,
    pub samples: &'a [i16],
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

    pub fn from_data(asset: super::DataAsset, data: CreationData) -> Self {
        Sfx {
            asset,
            len: data.len,
            loop_start: data.loop_start,
            loop_len: data.loop_len,
            bits_per_sample: data.bits_per_sample,
            samples: Vec::from(data.samples),
        }
    }
}

impl super::GenericAsset for Sfx {
    //fn asset(&self) -> &super::DataAsset { &self.asset }
    fn data_size(&self) -> usize {
        // header: len(4) + loop_start(4) + loop_len(4) + bits_per_sample(4) + samples<ptr>(4)
        let header = 4usize * 4usize + 4usize;

        // data: bits_per_sample/8 * len
        let data = (self.bits_per_sample as usize) / 8usize * self.samples.len();

        header + data
    }
}
