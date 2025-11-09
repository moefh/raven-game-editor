use egui::{Sense, Color32, Vec2};

pub struct SfxDisplayState {
    pub samples_per_point: f32,
    pub first_sample: f32,
}

impl SfxDisplayState {
    pub fn new() -> Self {
        SfxDisplayState {
            samples_per_point: 0.0,
            first_sample: 0.0,
        }
    }

    fn get_marker_pos(&self, sample_index: f32) -> f32 {
        (sample_index - self.first_sample) / self.samples_per_point
    }

    fn zoom_by(&mut self, delta: f32, center: f32, canvas_width: f32, samples: &[i16]) {
        let delta = if self.samples_per_point / delta < 0.125 {
            0.125 / self.samples_per_point
        } else {
            1.0 / delta
        };
        let center = self.first_sample + center * self.samples_per_point;
        self.samples_per_point *= delta;
        self.first_sample = (center + (self.first_sample - center) * delta).round();

        self.clip_scroll(canvas_width, samples);
    }

    fn clip_scroll(&mut self, canvas_width: f32, samples: &[i16]) {
        let canvas_width = canvas_width.floor() - 1.0;
        let samples_len = samples.len() as f32;
        if (samples_len - self.first_sample) / self.samples_per_point < canvas_width {
            self.first_sample = (samples_len - canvas_width * self.samples_per_point).round();
        }
        if self.first_sample < 0.0 { self.first_sample = 0.0; }
    }
}

pub fn sfx_display(ui: &mut egui::Ui, state: &mut SfxDisplayState, samples: &[i16], loop_start: &mut f32, loop_end: &mut f32) {
    let ask_size = Vec2::new(100.0, 50.0).max(ui.available_size());
    let (response, painter) = ui.allocate_painter(ask_size, Sense::drag());
    let canvas_rect = response.rect;

    painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::BLACK);
    let stroke = if state.samples_per_point > 2.0 {
        egui::Stroke::new(1.0, Color32::from_rgb(0x3f, 0x3f, 0x3f))
    } else {
        egui::Stroke::new(1.0, Color32::from_rgb(0xc0, 0xc0, 0xc0))
    };
    painter.hline(canvas_rect.x_range(), canvas_rect.min.y + canvas_rect.height()/2.0, stroke);

    if samples.is_empty() { return; }  // nothing to do if we have no samples!

    if state.samples_per_point == 0.0 {
        // wait until next frame so we have an accurate canvas width
        // before settling on the samples_per_point value:
        state.samples_per_point = 0.1;
    } else if state.samples_per_point < 0.125 {
        let samples_len = samples.len() as f32;
        state.samples_per_point = if samples_len > canvas_rect.width() {
            (samples_len / canvas_rect.width()).ceil()
        } else {
            1.0
        };
    }

    // limit scroll in case we've been resized
    state.clip_scroll(canvas_rect.width(), samples);

    let samples_height = canvas_rect.height();
    let samples_x_start = canvas_rect.min.x;
    let samples_y_start = canvas_rect.min.y + samples_height / 2.0;

    // draw samples
    let sample_stroke = egui::Stroke::new(1.0, Color32::WHITE);
    if state.samples_per_point >= 1.0 {
        for i in 0 .. (canvas_rect.width().ceil() as usize) {
            let start_index = ((state.first_sample + i as f32 * state.samples_per_point).round() as usize).min(samples.len()-1);
            let end_index = ((state.first_sample + (i+1) as f32 * state.samples_per_point).round() as usize).min(samples.len()-1);
            if start_index == end_index { break }
            let min_max = samples[start_index..end_index].iter().fold((i16::MAX, i16::MIN), |min_max, &sample| {
                let min = if min_max.0 > sample { sample } else { min_max.0 };
                let max = if min_max.1 < sample { sample } else { min_max.1 };
                (min, max)
            });
            let min = samples_y_start + (min_max.0 as f32) * samples_height / (i16::MAX as f32) * 0.45;
            let max = samples_y_start + (min_max.1 as f32) * samples_height / (i16::MAX as f32) * 0.45;
            painter.vline(samples_x_start + i as f32, min..=max, sample_stroke);
        }
    } else {
        let num_samples_to_draw = (canvas_rect.width() * state.samples_per_point).floor() as usize + 1;
        let first_index = state.first_sample.round() as usize;
        let mut points = vec![egui::Pos2::ZERO; num_samples_to_draw];
        for (i, point) in points.iter_mut().enumerate() {
            let sample = if first_index + i < samples.len() {
                samples[first_index + i]
            } else {
                0
            };
            point.x = samples_x_start + (i as f32) / state.samples_per_point;
            point.y = samples_y_start + (sample as f32) * samples_height / (i16::MAX as f32) * 0.45;
        }
        painter.line(points, sample_stroke);
    }

    // draw loop start marker
    let loop_start_pos = state.get_marker_pos(*loop_start);
    let loop_start_stroke = egui::Stroke::new(3.0, Color32::BLUE);
    painter.vline(samples_x_start + loop_start_pos, canvas_rect.y_range(), loop_start_stroke);

    // draw loop end marker
    let loop_end_pos = state.get_marker_pos(*loop_end);
    let loop_end_stroke = egui::Stroke::new(3.0, Color32::RED);
    painter.vline(samples_x_start + loop_end_pos, canvas_rect.y_range(), loop_end_stroke);

    // check zoom
    if let (true, Some(hover_pos)) = (
        response.contains_pointer(),
        ui.input(|i| i.pointer.hover_pos()),
    ) {
        let zoom_delta = ui.input(|i| i.zoom_delta());  // or use i.smooth_scroll_delta if CTRL key is should not be required?
        if zoom_delta != 1.0 {
            state.zoom_by(zoom_delta, hover_pos.x - canvas_rect.min.x, canvas_rect.width(), samples);
        }
    }

    // check pan
    if response.dragged_by(egui::PointerButton::Middle) {
        state.first_sample -= response.drag_delta().x * state.samples_per_point;
        state.first_sample = state.first_sample.round();
        state.clip_scroll(canvas_rect.width(), samples);
    }

    // check click
    if let Some(pointer_pos) = response.interact_pointer_pos() {
        let pos = ((pointer_pos.x - canvas_rect.min.x) * state.samples_per_point + state.first_sample).floor();
        if response.dragged_by(egui::PointerButton::Primary) {
            *loop_start = pos;
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            *loop_end = pos;
        }
    }
}
