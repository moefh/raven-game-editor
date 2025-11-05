
const MOD_PERIOD_TABLE : &[u16] = &[
    2*1712,2*1616,2*1524,2*1440,2*1356,2*1280,2*1208,2*1140,2*1076,2*1016,2*960,2*906,
    1712,1616,1524,1440,1356,1280,1208,1140,1076,1016,960,907,
    856,808,762,720,678,640,604,570,538,508,480,453,
    428,404,381,360,339,320,302,285,269,254,240,226,
    214,202,190,180,170,160,151,143,135,127,120,113,
    107,101,95,90,85,80,75,71,67,63,60,56,
    53,50,47,45,42,40,37,35,33,31,30,28,
];

#[allow(unused)]
pub struct ModSample {
    pub len: u32,
    pub loop_start: u32,
    pub loop_len: u32,
    pub finetune: i8,
    pub volume: u8,
    pub bits_per_sample: u16,
    pub data: Option<Vec<i16>>,
}

#[allow(unused)]
#[derive(Copy, Clone)]
pub struct ModCell {
    pub sample: u8,
    pub period: u16,
    pub effect: u16,
}

#[allow(unused)]
pub struct ModData {
    pub asset: super::DataAsset,
    pub samples: Vec<ModSample>,
    pub pattern: Vec<ModCell>,
    pub song_positions: Vec<u8>,
    pub num_channels: u8,
}

impl ModData {

    pub fn new(asset: super::DataAsset) -> Self {
        ModData {
            asset,
            num_channels: 0,
            samples: Vec::new(),
            pattern: Vec::new(),
            song_positions: Vec::new(),
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
    
}
