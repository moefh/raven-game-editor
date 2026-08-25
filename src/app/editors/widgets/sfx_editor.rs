use egui::{
    Sense,
    Color32,
    Pos2,
    Vec2,
    Rect,
};

const MIN_SAMPLES_PER_POINT: f32 = 0.125;

#[derive(Clone, Copy, PartialEq)]
pub enum SfxTool {
    Select,
    SetLoop,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SfxEditorAction {
    Select,
    Delete,
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct SfxSelection {
    pub start: u32,
    pub end: u32,
}

impl SfxSelection {
    pub fn new(start: u32, end: u32) -> Self {
        SfxSelection {
            start: start.min(end),
            end: start.max(end),
        }
    }

    pub fn set(&mut self, start: u32, end: u32) {
        self.start = start.min(end);
        self.end = start.max(end);
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn contains(&self, index: u32) -> bool {
        self.start <= index && index <= self.end
    }
}

pub struct SfxEditorWidget {
    pub samples_per_point: f32,
    pub first_sample: f32,
    pub tool_mouse_down: bool,
    pub tool: SfxTool,
    pub selection_enabled: bool,
    pub selection: Option<SfxSelection>,
    drag_start_sample_index: u32,
}

impl SfxEditorWidget {
    pub fn new() -> Self {
        SfxEditorWidget {
            samples_per_point: 100.0,
            first_sample: 0.0,
            tool_mouse_down: false,
            tool: SfxTool::Select,
            selection_enabled: true,
            selection: None,
            drag_start_sample_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.samples_per_point = 100.0;
        self.first_sample = 0.0;
        self.tool_mouse_down = false;
        self.selection = None;
        self.drag_start_sample_index = 0;
    }

    pub fn set_tool(&mut self, tool: SfxTool) {
        self.tool = tool;
    }

    fn get_sample_pos(&self, sample_index: u32) -> f32 {
        (sample_index as f32 - self.first_sample) / self.samples_per_point
    }

    fn zoom_by(&mut self, delta: f32, center: f32, canvas_width: f32, num_samples: usize) {
        let delta = if self.samples_per_point / delta < MIN_SAMPLES_PER_POINT {
            MIN_SAMPLES_PER_POINT / self.samples_per_point
        } else {
            1.0 / delta
        };

        let center = self.first_sample + center * self.samples_per_point;
        self.samples_per_point *= delta;
        self.first_sample = (center + (self.first_sample - center) * delta).round();

        self.clip_scroll(canvas_width, num_samples);
    }

    fn clip_scroll(&mut self, canvas_width: f32, num_samples: usize) {
        let num_samples = num_samples as f32;

        // don't let samples zoom out so much that they don't cover the whole canvas
        if canvas_width > 10.0 &&
            self.samples_per_point > num_samples / canvas_width &&
            num_samples / canvas_width > MIN_SAMPLES_PER_POINT {
                self.samples_per_point = num_samples / canvas_width;
            }

        let canvas_width = canvas_width.floor() - 1.0;
        if (num_samples - self.first_sample) / self.samples_per_point < canvas_width {
            self.first_sample = (num_samples - canvas_width * self.samples_per_point).round();
        }
        if self.first_sample < 0.0 { self.first_sample = 0.0; }
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, samples: &mut Vec<i16>, loop_start: &mut u32, loop_end: &mut u32) -> SfxEditorAction {
        if self.selection_enabled {
            let cmd_a = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::A);
            if ui.input_mut(|i| i.consume_shortcut(&cmd_a)) {
                let samples_len = samples.len() as u32;
                if self.selection.is_some_and(|sel| sel.start == 0 && sel.end == samples_len) {
                    self.selection = None;
                } else {
                    self.selection = Some(SfxSelection::new(0, samples_len));
                }
                return SfxEditorAction::Select;
            }
        }

        let del = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete);
        if ui.input_mut(|i| i.consume_shortcut(&del)) && let Some(selection) = self.selection && ! selection.is_empty() {
            let start = (selection.start as usize).clamp(0, samples.len());
            let end = (selection.end as usize).clamp(0, samples.len());
            if start < end {
                samples.splice(start..end, []);
                if selection.contains(*loop_start) {
                    *loop_start = selection.start;
                } else if *loop_start > selection.end {
                    *loop_start -= selection.len();
                }
                if selection.contains(*loop_end) {
                    *loop_end = selection.start;
                } else if *loop_end > selection.end {
                    *loop_end -= selection.len();
                }
                self.selection = None;
                return SfxEditorAction::Delete;
            }
        }

        SfxEditorAction::None
    }

    fn handle_selection_mouse(&mut self, response: &egui::Response, keys_pressed: egui::Modifiers, mouse_sample_index: u32) {
        if response.dragged_by(egui::PointerButton::Primary) {
            if response.drag_started() {
                if keys_pressed.shift && let Some(selection) = &mut self.selection && ! selection.is_empty() {
                    if mouse_sample_index > selection.start {
                        self.drag_start_sample_index = selection.start;
                    } else {
                        self.drag_start_sample_index = selection.end;
                    }
                    selection.set(mouse_sample_index, self.drag_start_sample_index);
                } else {
                    self.drag_start_sample_index = mouse_sample_index;
                }
            } else if let Some(selection) = &mut self.selection {
                selection.set(mouse_sample_index, self.drag_start_sample_index);
            } else {
                self.selection = Some(SfxSelection::new(mouse_sample_index, self.drag_start_sample_index));
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, samples: &[i16], loop_start: &mut u32, loop_end: &mut u32, height: f32) {
        let ask_size = if height <= 0.0 {
            Vec2::new(100.0, 50.0).max(ui.available_size())
        } else {
            Vec2::new(100.0, 50.0).max(Vec2::new(ui.available_size().x, height))
        };
        let (response, painter) = ui.allocate_painter(ask_size, Sense::drag());
        let canvas_rect = response.rect;

        painter.rect_filled(canvas_rect, egui::CornerRadius::ZERO, Color32::BLACK);

        let num_samples = samples.len();
        if num_samples == 0 || num_samples > u32::MAX as usize { return; }

        // limit scroll in case we've been resized
        self.clip_scroll(canvas_rect.width(), num_samples);

        let samples_height = canvas_rect.height();
        let samples_x_start = canvas_rect.min.x;
        let samples_y_start = canvas_rect.min.y + samples_height / 2.0;

        // draw selection
        if let Some(selection) = self.selection {
            let sel_rect = Rect::from_min_max(
                Pos2::new(samples_x_start + self.get_sample_pos(selection.start), canvas_rect.min.y),
                Pos2::new(samples_x_start + self.get_sample_pos(selection.end), canvas_rect.max.y),
            );
            painter.rect_filled(sel_rect, egui::CornerRadius::ZERO, Color32::from_rgb(0, 0, 64));
        }

        // draw line at zero level
        let stroke = if self.samples_per_point > 2.0 || self.samples_per_point <= 0.1 {
            egui::Stroke::new(1.0, Color32::from_rgb(0x3f, 0x3f, 0x3f))
        } else {
            egui::Stroke::new(1.0, Color32::from_rgb(0xc0, 0xc0, 0xc0))
        };
        painter.hline(canvas_rect.x_range(), canvas_rect.min.y + canvas_rect.height()/2.0, stroke);

        // draw samples
        let sample_stroke = egui::Stroke::new(1.0, Color32::WHITE);
        if self.samples_per_point >= 1.0 {
            for i in 0 .. (canvas_rect.width().ceil() as usize) {
                let start_index = ((self.first_sample + i as f32 * self.samples_per_point).round() as usize).min(num_samples-1);
                let end_index = ((self.first_sample + (i+1) as f32 * self.samples_per_point).round() as usize).min(num_samples-1);
                if start_index == end_index { break }
                let min_max = samples[start_index..end_index].iter().fold((i16::MAX, i16::MIN), |min_max, &sample| {
                    let min = if min_max.0 > sample { sample } else { min_max.0 };
                    let max = if min_max.1 < sample { sample } else { min_max.1 };
                    (min, max)
                });
                let min = samples_y_start - (min_max.0 as f32) * samples_height / (i16::MAX as f32) * 0.45;
                let max = samples_y_start - (min_max.1 as f32) * samples_height / (i16::MAX as f32) * 0.45;
                painter.vline(samples_x_start + i as f32, min..=max, sample_stroke);
            }
        } else {
            let num_samples_to_draw = (canvas_rect.width() * self.samples_per_point).floor() as usize + 1;
            let first_index = self.first_sample.round() as usize;
            let mut points = vec![egui::Pos2::ZERO; num_samples_to_draw];
            for (i, point) in points.iter_mut().enumerate() {
                let sample = if first_index + i < num_samples {
                    samples[first_index + i]
                } else {
                    0
                };
                point.x = samples_x_start + (i as f32) / self.samples_per_point;
                point.y = samples_y_start - (sample as f32) * samples_height / (i16::MAX as f32) * 0.45;
            }
            painter.line(points, sample_stroke);
        }

        // draw loop start marker
        let loop_start_pos = self.get_sample_pos(*loop_start);
        let loop_start_stroke = egui::Stroke::new(3.0, Color32::BLUE);
        painter.vline(samples_x_start + loop_start_pos, canvas_rect.y_range(), loop_start_stroke);

        // draw loop end marker
        let loop_end_pos = self.get_sample_pos(*loop_end);
        let loop_end_stroke = egui::Stroke::new(3.0, Color32::RED);
        painter.vline(samples_x_start + loop_end_pos, canvas_rect.y_range(), loop_end_stroke);

        // draw parts of start marker again (in case it's under loop end)
        if (loop_start_pos - loop_end_pos).abs() < 3.0 {
            let num_frags = 4;
            for i in 0..num_frags {
                let frag = canvas_rect.height() / (2.0 * num_frags as f32);
                let start = canvas_rect.min.y + frag * (2*i + 1) as f32;
                let end = canvas_rect.min.y + frag * (2*i + 2) as f32;
                painter.vline(samples_x_start + loop_start_pos, start..=end, loop_start_stroke);
            }
        }

        // ====================================================
        // == handle input

        let keys_pressed = ui.ctx().input(|i| i.modifiers);

        // check hover
        if response.contains_pointer() && response.hovered() {
            if keys_pressed.alt {
                if response.dragged() {
                    response.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                } else {
                    response.ctx.set_cursor_icon(egui::CursorIcon::Grab);
                }
            } else if keys_pressed.command {
                response.ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
            }
        }

        // check zoom
        if response.contains_pointer() && let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let zoom_delta = if keys_pressed.command && response.dragged_by(egui::PointerButton::Primary) {
                (response.drag_delta().y * -0.01).exp()
            } else {
                ui.input(|i| i.zoom_delta())
            };
            if zoom_delta != 1.0 {
                self.zoom_by(zoom_delta, hover_pos.x - canvas_rect.min.x, canvas_rect.width(), num_samples);
            }
        }

        // check pan
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged() && keys_pressed.alt) {
            self.first_sample -= response.drag_delta().x * self.samples_per_point;
            self.first_sample = self.first_sample.round();
            self.clip_scroll(canvas_rect.width(), num_samples);
        }

        // check click
        if response.drag_stopped() {
            self.tool_mouse_down = false;
        }
        if (self.tool_mouse_down || response.drag_started()) &&
            let Some(pointer_pos) = response.interact_pointer_pos() &&
            ! (keys_pressed.alt || keys_pressed.command) {
                if response.drag_started() {
                    self.tool_mouse_down = true;
                }
                let pos = ((pointer_pos.x - canvas_rect.min.x) * self.samples_per_point + self.first_sample).floor();
                let mouse_sample_index = pos.clamp(0.0, num_samples as f32) as u32;
                match self.tool {
                    SfxTool::Select => {
                        if self.selection_enabled {
                            let mouse_sample_index = if pointer_pos.y < canvas_rect.min.y {
                                0
                            } else if pointer_pos.y > canvas_rect.max.y {
                                num_samples as u32
                            } else {
                                mouse_sample_index
                            };
                            self.handle_selection_mouse(&response, keys_pressed, mouse_sample_index);
                        }
                    }
                    SfxTool::SetLoop => {
                        if response.dragged_by(egui::PointerButton::Primary) {
                            *loop_start = mouse_sample_index;
                            if *loop_end < *loop_start { *loop_end = *loop_start; }
                        } else if response.dragged_by(egui::PointerButton::Secondary) {
                            *loop_end = mouse_sample_index;
                            if *loop_start > *loop_end { *loop_start = *loop_end; }
                        }
                    }
                }
            }
    }
}
