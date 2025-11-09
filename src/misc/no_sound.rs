#[allow(dead_code)]
pub struct SoundPlayer {
    pub error: Option<String>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        SoundPlayer {
            error: Some("sound disabled during compilation".to_string()),
        }
    }

    pub fn is_available(&self) -> bool {
        false
    }

    pub fn play_s16(&mut self, _samples: &[i16], _freq: f32, _volume: f32) -> bool {
        false
    }
}
