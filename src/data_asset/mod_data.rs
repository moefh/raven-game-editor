pub const MOD_PERIOD_TABLE : &[u16] = &[
    2*1712,2*1616,2*1524,2*1440,2*1356,2*1280,2*1208,2*1140,2*1076,2*1016,2*960,2*906,
    1712,1616,1524,1440,1356,1280,1208,1140,1076,1016,960,907,
    856,808,762,720,678,640,604,570,538,508,480,453,
    428,404,381,360,339,320,302,285,269,254,240,226,
    214,202,190,180,170,160,151,143,135,127,120,113,
    107,101,95,90,85,80,75,71,67,63,60,56,
    53,50,47,45,42,40,37,35,33,31,30,28,
];

#[derive(std::hash::Hash)]
pub struct ModSample {
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub finetune: i8,
    pub volume: u8,
    pub bits_per_sample: u16,
    pub data: Option<Vec<i16>>,
}

#[derive(Copy, Clone, std::hash::Hash)]
pub struct ModCell {
    pub sample: u8,
    pub period: u16,
    pub effect: u16,
}

#[derive(std::hash::Hash)]
pub struct ModData {
    pub asset: super::DataAsset,
    pub samples: Vec<ModSample>,
    pub pattern: Vec<ModCell>,
    pub song_positions: Vec<u8>,
    pub num_channels: u8,
}

pub struct CreationData<'a> {
    pub samples: Vec<ModSample>,
    pub pattern: &'a [ModCell],
    pub song_positions: Vec<u8>,
    pub num_channels: u8,
}

impl ModData {
    pub fn new(id: super::DataAssetId, name: String) -> Self {
        let num_channels = 4;
        ModData {
            asset: super::DataAsset::new(super::DataAssetType::ModData, id, name),
            num_channels,
            samples: Self::gen_samples(),
            pattern: Self::gen_pattern(num_channels),
            song_positions: vec![0; 1],
        }
    }

    pub fn from_data(id: super::DataAssetId, name: String, data: CreationData) -> Self {
        ModData {
            asset: super::DataAsset::new(super::DataAssetType::ModData, id, name),
            num_channels: data.num_channels,
            samples: data.samples,
            pattern: Vec::from(data.pattern),
            song_positions: data.song_positions,
        }
    }

    pub fn get_note_period(note: i32, octave: i32) -> u16 {
        let index = (octave * 12 + note) as usize;
        if index >= MOD_PERIOD_TABLE.len() {
            0
        } else {
            MOD_PERIOD_TABLE[index]
        }
    }

    pub fn get_period_note(period: u16) -> (i32, i32) {
        for octave in 0..MOD_PERIOD_TABLE.len()/12 {
            for note in 0..12 {
                let index = octave * 12 + note;
                if period >= MOD_PERIOD_TABLE[index] {
                    return (note as i32, octave as i32);
                }
            }
        }
        (-1,-1)
    }

    fn gen_samples() -> Vec<ModSample> {
        let mut samples = Vec::with_capacity(31);
        let sample_data_len = 5000;
        samples.push(ModSample {
            len: sample_data_len,
            loop_start: 0,
            loop_len: 0,
            finetune: 0,
            volume: 0x40,
            bits_per_sample: 16,
            data: Some(super::Sfx::gen_sample_data(sample_data_len as usize)),
        });
        for _ in 0..30 {
            samples.push(ModSample {
                len: 0,
                loop_start: 0,
                loop_len: 0,
                finetune: 0,
                volume: 0,
                bits_per_sample: 8,
                data: None,
            });
        }
        samples
    }

    fn gen_pattern(num_channels: u8) -> Vec<ModCell> {
        let num_cells = 64 * num_channels as usize;
        let mut pattern = vec![ModCell { sample: 0, period: 0, effect: 0 }; num_cells];
        pattern[0] = ModCell { sample: 1, period: Self::get_note_period(0, 3), effect: 0 };
        pattern[1] = ModCell { sample: 1, period: Self::get_note_period(4, 3), effect: 0 };
        pattern[2] = ModCell { sample: 1, period: Self::get_note_period(7, 3), effect: 0 };
        pattern[3] = ModCell { sample: 0, period: 0, effect: 0xF78 };
        pattern[16] = ModCell { sample: 0, period: 0, effect: 0xD00 };
        pattern
    }
}

impl super::GenericAsset for ModData {
    fn asset(&self) -> &super::DataAsset { &self.asset }

    fn data_size(&self) -> usize {
        // sample_header: len(4) + loop_start(4) + loop_len(4) +
        //                finetune(1) + volume(1) + bits_per_sample(2) + data<ptr>(4)
        let sample_header = 3usize * 4usize + 4usize + 4usize;

        // header: 31*sample_header + num_channels(1) +
        //         num_song_positions(1) + song_positions(128) +
        //         num_patterns(1) + pad(1) + pattern<ptr>(4)
        let header = 31usize * sample_header + 132usize + 4usize;

        // cell_header: sample(1) + note_index(1) + effect(2)
        let cell_header = 1usize + 1usize + 2usize;

        // patterns: num_patterns * num_channels * 64 * cell_header
        let patterns = self.pattern.len() * cell_header;

        // samples:
        let samples = self.samples.iter().fold(0, |acc, s| acc + s.len as usize);

        header + patterns + samples
    }
}
