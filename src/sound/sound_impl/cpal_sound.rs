use std::result::Result;
use std::error::Error;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::super::player::Player;

const USE_FILTER: bool = true;
const MIN_SAMPLE_RATE: u32 = 44100;
const MAX_SAMPLE_RATE: u32 = 48000;
const REQ_BUFFER_SIZE: u32 = 4096;

pub struct SoundPlayerImpl {
    pub name: String,
    _host: cpal::Host,
    _device: cpal::Device,
    stream: cpal::Stream,
    player: Arc<Mutex<Player>>,
}

impl SoundPlayerImpl {
    fn read_supported_output_configs(device: &cpal::Device) -> String {
        let configs_iter = device.supported_output_configs();
        match configs_iter {
            Ok(configs) => {
                let mut s = String::new();
                for config in configs {
                    s.push_str(&format!("-> {:?}\n", config));
                }
                s
            }
            Err(e) => e.to_string()
        }
    }

    fn find_preferred_config(device: &cpal::Device) -> Result<Option<cpal::SupportedStreamConfigRange>, Box<dyn Error>> {
        let configs = device.supported_output_configs()?.find(|range| {
            let min_sample_rate = MIN_SAMPLE_RATE.max(range.min_sample_rate().0);
            let max_sample_rate = MAX_SAMPLE_RATE.min(range.max_sample_rate().0);
            if matches!(range.sample_format(), cpal::SampleFormat::I16) &&
                range.channels() == 1 &&
                min_sample_rate <= max_sample_rate &&
                let cpal::SupportedBufferSize::Range{ min: min_buffer_size, max: max_buffer_size } = range.buffer_size() &&
                *min_buffer_size <= REQ_BUFFER_SIZE &&
                *max_buffer_size >= REQ_BUFFER_SIZE {
                    true
                } else {
                    false
                }
        });
        Ok(configs)
    }

    fn find_acceptable_config(device: &cpal::Device) -> Result<Option<cpal::SupportedStreamConfigRange>, Box<dyn Error>> {
        let configs = device.supported_output_configs()?.find(|range| {
            let min_sample_rate = MIN_SAMPLE_RATE.max(range.min_sample_rate().0);
            let max_sample_rate = MAX_SAMPLE_RATE.min(range.max_sample_rate().0);
            if matches!(range.sample_format(), cpal::SampleFormat::I16) &&
                range.channels() <= 2 &&
                min_sample_rate <= max_sample_rate &&
                let cpal::SupportedBufferSize::Range{ min: min_buffer_size, max: max_buffer_size } = range.buffer_size() &&
                *min_buffer_size <= REQ_BUFFER_SIZE &&
                *max_buffer_size >= REQ_BUFFER_SIZE {
                    true
                } else {
                    false
                }
        });
        Ok(configs)
    }

    fn open_sound() -> Result<SoundPlayerImpl, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| {
            std::io::Error::other("can't open audio output device")
        })?;
        let config_range = match Self::find_preferred_config(&device)? {
            Some(config_range) => Some(config_range),
            None => Self::find_acceptable_config(&device)?,
        };
        let config_range = config_range.ok_or_else(|| {
            std::io::Error::other(format!("no suitable config found.\nSupported configs:\n{}",
                                          Self::read_supported_output_configs(&device)))
        })?;
        let min_sample_rate = config_range.min_sample_rate().0;
        let max_sample_rate = config_range.max_sample_rate().0;
        let sample_rate = MAX_SAMPLE_RATE.clamp(min_sample_rate, max_sample_rate);
        let mut config = config_range.try_with_sample_rate(cpal::SampleRate(sample_rate)).ok_or_else(|| {
            std::io::Error::other("sample rate not supported")
        })?.config();
        config.buffer_size = cpal::BufferSize::Fixed(REQ_BUFFER_SIZE);

        let player = Arc::new(Mutex::new(Player::new(config.channels as usize, config.sample_rate.0 as f32, USE_FILTER)));
        let player_clone = player.clone();
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut player = player_clone.lock().unwrap();
                player.render_samples(data);
            },
            move |err| { println!("CPAL error: {}", err); },
            None)?;
        stream.pause()?;

        Ok(SoundPlayerImpl {
            name: format!("cpal at {} Hz, {} channel(s), filter {}",
                          config.sample_rate.0, config.channels,
                          if USE_FILTER { "enabled" } else { "disabled" }),
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
