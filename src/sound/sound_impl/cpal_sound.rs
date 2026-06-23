use std::result::Result;
use std::error::Error;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::super::player::Player;

const USE_FILTER: bool = true;
const PREFERRED_BUFFER_SIZE: u32 = 4096;

pub struct SoundPlayerImpl {
    pub name: String,
    _host: cpal::Host,
    _device: cpal::Device,
    stream: cpal::Stream,
    player: Arc<Mutex<Player>>,
}

impl SoundPlayerImpl {
    fn open_sound() -> Result<SoundPlayerImpl, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| {
            std::io::Error::other("can't open audio output device")
        })?;
        let supported_config = device.default_output_config()?;
        let config = if let cpal::SupportedBufferSize::Range { min, max } = supported_config.buffer_size() {
            // if we get a range of possible buffer sizes, choose the closest to the preferred one
            let mut config: cpal::StreamConfig = supported_config.into();
            config.buffer_size = cpal::BufferSize::Fixed(PREFERRED_BUFFER_SIZE.clamp(*min, *max));
            config
        } else {
            supported_config.into()
        };
        let player = Arc::new(Mutex::new(Player::new(config.channels as usize, config.sample_rate as f32, USE_FILTER)));
        let player_clone = player.clone();
        let stream = device.build_output_stream(
            config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut player = player_clone.lock().unwrap();
                player.render_samples(data);
            },
            move |err| { println!("CPAL error: {}", err); },
            None
        )?;

        let name = format!(
            "cpal at {} Hz, {} channel(s), buffer size {}, filter {}",
            config.sample_rate,
            config.channels,
            if let cpal::BufferSize::Fixed(size) = config.buffer_size { format!("{}", size) } else { "unknown".to_owned() },
            if USE_FILTER { "enabled" } else { "disabled" }
        );
        Ok(SoundPlayerImpl {
            name,
            _host: host,
            _device: device,
            stream,
            player,
        })
    }

    pub fn start() -> Result<Self, String> {
        Self::open_sound().map_err(|e| e.to_string())
    }

    pub fn play_s16(&mut self, samples: &[i16], freq: f32, volume: f32) {
        {
            let mut player = self.player.lock().unwrap();
            player.setup(samples, freq, volume);
        }
        self.stream.play().unwrap_or(());
    }
}
