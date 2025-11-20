use std::result::Result;
use soloud::AudioExt;

pub struct SoundPlayerImpl {
    pub name: String,
    soloud: soloud::Soloud,
    wav: soloud::audio::Wav,
}

impl SoundPlayerImpl {
    pub fn start() -> Result<Self, String> {
        let soloud = soloud::Soloud::default().map_err(|e| format!("error initializing soloud: {}", e.to_string()))?;
        let wav = soloud::audio::Wav::default();
        Ok(SoundPlayerImpl {
            name: "soloud".to_string(),
            soloud,
            wav,
        })
    }

    pub fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) {
        self.wav.stop();
        self.soloud.set_global_volume(volume);
        unsafe {
            if self.wav.load_raw_wav_16_ex(samples, freq, 1).is_err() {
                return;
            }
        }
        self.soloud.play(&self.wav);
    }
}
