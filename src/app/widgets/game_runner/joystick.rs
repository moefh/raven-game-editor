
pub const JOY_NONE: u32   = 0;
pub const JOY_UP: u32     = 1 << 0;
pub const JOY_DOWN: u32   = 1 << 1;
pub const JOY_LEFT: u32   = 1 << 2;
pub const JOY_RIGHT: u32  = 1 << 3;
pub const JOY_X: u32      = 1 << 4;
pub const JOY_Y: u32      = 1 << 5;
pub const JOY_A: u32      = 1 << 6;
pub const JOY_B: u32      = 1 << 7;

pub struct Joystick {
    cur: u32,
    last: u32,
}

impl Joystick {
    pub fn new() -> Self {
        Joystick {
            cur: 0,
            last: 0,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        self.last = self.cur;
        ui.input(|i| {
            self.cur = JOY_NONE |
                if i.key_down(egui::Key::ArrowUp) { JOY_UP } else { 0 } |
                if i.key_down(egui::Key::ArrowDown) { JOY_DOWN } else { 0 } |
                if i.key_down(egui::Key::ArrowLeft) { JOY_LEFT } else { 0 } |
                if i.key_down(egui::Key::ArrowRight) { JOY_RIGHT } else { 0 } |
                if i.key_down(egui::Key::Space) { JOY_A } else { 0 };
                if i.key_down(egui::Key::B) { JOY_B } else { 0 };
                if i.key_down(egui::Key::X) { JOY_X } else { 0 };
                if i.key_down(egui::Key::Y) { JOY_Y } else { 0 };
        });
    }

    pub fn held(&self, buttons: u32) -> bool {
        (self.cur & buttons) != 0
    }

    pub fn pressed(&self, buttons: u32) -> bool {
        (self.last & buttons) == 0 && (self.cur & buttons) != 0
    }
}
