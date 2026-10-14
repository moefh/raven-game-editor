use std::result::Result;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = play_sound)]
    fn play_sound(samples: Vec<i16>, freq: f32, volume: f32);
}

pub struct SoundPlayerImpl {
    pub name: String,
}

impl SoundPlayerImpl {
    pub fn start() -> Result<Self, String> {
        Ok(SoundPlayerImpl {
            name: "browser's Web Audio API".to_owned(),
        })
    }

    pub fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) {
        play_sound(samples.to_vec(), freq, volume);
    }
}
