use std::result::Result;
use std::error::Error;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const USE_FILTER: bool = true;
const PREFERRED_BUFFER_SIZE: u32 = 4096;

#[allow(dead_code)]
struct Player {
    data: Vec<i16>,
    num_channels: usize,
    sample_rate: f32,
    play_pos: f32,
    play_step: f32,
    play_volume: f32,
    use_filter: bool,
}

#[allow(dead_code)]
impl Player {
    pub fn new(num_channels: usize, sample_rate: f32, use_filter: bool) -> Self {
        Player {
            data: Vec::new(),
            play_pos: 0.0,
            play_step: 0.0,
            play_volume: 0.0,
            num_channels,
            sample_rate,
            use_filter,
        }
    }

    pub fn setup(&mut self, data: &[i16], freq: f32, volume: f32) {
        self.data.clear();
        self.data.extend_from_slice(data);
        self.play_pos = 0.0;
        self.play_step = freq / self.sample_rate;
        self.play_volume = volume;
    }

    fn render_samples_raw(&mut self, data: &mut [i16]) {
        let mut play_pos = self.play_pos;
        let play_step = self.play_step;
        for spl in data.chunks_exact_mut(self.num_channels) {
            let spl_index = play_pos.round() as usize;
            let out_spl = if spl_index >= self.data.len() {
                0
            } else {
                (self.data[spl_index] as f32 * self.play_volume).clamp(i16::MIN as f32, i16::MAX as f32) as i16
            };
            play_pos += play_step;
            for s in spl.iter_mut().take(self.num_channels) {
                *s = out_spl;
            }
        }
        self.play_pos = play_pos;
    }

    pub fn render_samples(&mut self, data: &mut [i16]) {
        if self.play_step >= 1.0 || ! self.use_filter {
            self.render_samples_raw(data);
            return;
        }

        let mut play_pos = self.play_pos;
        let play_step = self.play_step;

        for spl in data.chunks_exact_mut(self.num_channels) {
            let play_pos_floor = play_pos.floor();
            let play_pos_frac = play_pos - play_pos_floor;
            let last_index = play_pos_floor as usize;
            let next_index = play_pos.ceil() as usize;
            let out_spl = if last_index >= self.data.len() || next_index >= self.data.len() {
                0
            } else {
                let last = (self.data[last_index] as f32 * self.play_volume).clamp(i16::MIN as f32, i16::MAX as f32);
                let next = (self.data[next_index] as f32 * self.play_volume).clamp(i16::MIN as f32, i16::MAX as f32);
                (last * (1.0 - play_pos_frac) + next * play_pos_frac) as i16
            };
            play_pos += play_step;
            for s in spl.iter_mut().take(self.num_channels) {
                *s = out_spl;
            }
        }
        self.play_pos = play_pos;
    }
}

pub struct SoundPlayerImpl {
    pub name: String,
    _host: cpal::Host,
    _device: cpal::Device,
    stream: cpal::Stream,
    player: Arc<Mutex<Player>>,
}

impl SoundPlayerImpl {
    fn open_sound() -> Result<Self, Box<dyn Error>> {
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
