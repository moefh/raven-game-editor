use std::io::{Result, Error};
use soloud::AudioExt;

struct SoloudPlayer {
    soloud: soloud::Soloud,
    wav: soloud::audio::Wav,
}

impl SoloudPlayer {
    fn new() -> Result<Self> {
        let soloud = match soloud::Soloud::default() {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::other(format!("ERROR initializing Soloud: {}", e)));
            },
        };

        let wav = soloud::audio::Wav::default();

        Ok(SoloudPlayer {
            soloud,
            wav,
        })
    }

    fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) -> Result<()> {
        self.wav.stop();
        self.soloud.set_global_volume(volume);
        unsafe {
            if let Err(e) = self.wav.load_raw_wav_16_ex(samples, freq, 1) {
                return Err(Error::other(format!("Error loading samples: {}", e)));
            }
        }
        self.soloud.play(&self.wav);
        Ok(())
    }

}

pub struct SoundPlayer {
    pub error: Option<String>,
    player: Option<SoloudPlayer>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let (player, error) = match SoloudPlayer::new() {
            Ok(player) => (Some(player), None),
            Err(e) => (None, Some(format!("{}", e))),
        };

        SoundPlayer {
            player,
            error,
        }
    }

    pub fn is_available(&self) -> bool {
        self.player.is_some()
    }

    pub fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) -> bool {
        if let Some(player) = &mut self.player && let Err(e) = player.play_s16(samples, freq, volume) {
            self.error = Some(format!("{}", e));
            return false;
        }
        true
    }
}
