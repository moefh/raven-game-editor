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
        let len = 5000;
        Sfx {
            asset,
            len,
            loop_start: 0,
            loop_len: 0,
            bits_per_sample: 16,
            samples: Self::gen_sample_data(len as usize),
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

    pub fn gen_sample_data(len: usize) -> Vec<i16> {
        let mut samples = Vec::with_capacity(len);
        let k = 261.63 * 2.0 * std::f32::consts::PI / 22050.0;
        for i in 0..len {
            let osc1 = (i as f32 * k).sin() * 1.5;
            let osc2 = (i as f32 * k * 2.0).sin() * 1.1;
            let osc3 = (i as f32 * k * 3.0).sin() * 0.9;
            let env1 = (len - i) as f32 * i16::MAX as f32 / (len + 1) as f32;
            let env2 = (-2.0 * (i as f32) / len as f32).exp();
            samples.push((osc1 * osc2 * osc3 * env1 * env2) as i16);
        }
        samples
    }

}

impl super::GenericAsset for Sfx {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // header: len(4) + loop_start(4) + loop_len(4) + bits_per_sample(4) + samples<ptr>(4)
        let header = 4usize * 4usize + 4usize;

        // data: bits_per_sample/8 * len
        let data = (self.bits_per_sample as usize) / 8usize * self.samples.len();

        header + data
    }
}
