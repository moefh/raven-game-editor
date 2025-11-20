use std::result::Result;

pub struct SoundPlayerImpl {
    pub name: String,
}

impl SoundPlayerImpl {
    pub fn start() -> Result<Self, String> {
        Err("sound disabled during compilation".to_owned())
    }

    pub fn play_s16(&mut self, _samples: &[i16], _freq: f32, _volume: f32) {
    }
}
