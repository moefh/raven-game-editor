#[allow(dead_code)]
pub struct Player {
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
