use std::result::Result;

pub mod player;

// use no_sound if it's selected or if no other sound is selected
#[cfg(any(feature="no_sound", not(any(feature = "cpal_sound", feature="soloud_sound"))))]
mod sound_impl {
    mod no_sound;
    pub use no_sound::SoundPlayerImpl;
}

// use soloud if it's selected and no_sound is not (we prefer soloud over cpal because cpal is the default)
#[cfg(all(feature = "soloud_sound", not(feature = "no_sound")))]
mod sound_impl {
    mod soloud_sound;
    pub use soloud_sound::SoundPlayerImpl;
}

// use cpal if it's selected and no other sound is selected (that's the default)
#[cfg(all(feature = "cpal_sound", not(feature = "soloud_sound"), not(feature="no_sound")))]
mod sound_impl {
    mod cpal_sound;
    pub use cpal_sound::SoundPlayerImpl;
}

pub struct SoundPlayer {
    player: Result<sound_impl::SoundPlayerImpl, String>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        SoundPlayer {
            player: sound_impl::SoundPlayerImpl::start(),
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
