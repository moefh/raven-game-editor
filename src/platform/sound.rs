
use super::SoundPlayerImpl;

pub struct SoundPlayer {
    player: Result<SoundPlayerImpl, String>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        SoundPlayer {
            player: SoundPlayerImpl::start(),
        }
    }

    pub fn init_info(&self) -> String {
        match &self.player {
            Ok(player) => format!("Sound initialized: using {}", player.name),
            Err(error) => format!("No sound available: {}", error),
        }
    }

    pub fn is_available(&self) -> bool {
        self.player.is_ok()
    }

    pub fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) {
        if let Ok(player) = &mut self.player {
            player.play_s16(samples, freq, volume);
        }
    }
}
