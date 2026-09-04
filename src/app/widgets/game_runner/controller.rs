use crate::platform::update_gamepads;
pub use crate::platform::gamepad::{*};

use super::WindowContext;

pub struct Controller {
    cur: u32,
    old: u32,
    gamepads: Vec<Gamepad>,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            cur: 0,
            old: 0,
            gamepads: Vec::new(),
        }
    }

    fn read_keyboard(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            self.cur = GAMEPAD_NONE |
                if i.key_down(egui::Key::W) { GAMEPAD_UP } else { 0 } |
                if i.key_down(egui::Key::S) { GAMEPAD_DOWN } else { 0 } |
                if i.key_down(egui::Key::A) { GAMEPAD_LEFT } else { 0 } |
                if i.key_down(egui::Key::D) { GAMEPAD_RIGHT } else { 0 } |
                if i.key_down(egui::Key::ArrowUp) { GAMEPAD_UP } else { 0 } |
                if i.key_down(egui::Key::ArrowDown) { GAMEPAD_DOWN } else { 0 } |
                if i.key_down(egui::Key::ArrowLeft) { GAMEPAD_LEFT } else { 0 } |
                if i.key_down(egui::Key::ArrowRight) { GAMEPAD_RIGHT } else { 0 } |
                if i.key_down(egui::Key::Space) { GAMEPAD_SNES_B } else { 0 } |
                if i.key_down(egui::Key::Z) { GAMEPAD_SNES_A } else { 0 } |
                if i.key_down(egui::Key::X) { GAMEPAD_SNES_Y } else { 0 } |
                if i.key_down(egui::Key::C) { GAMEPAD_SNES_X } else { 0 };
        });
    }

    fn read_gamepads(&mut self) {
        for gp in &self.gamepads {
            self.cur |= gp.cur;
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, wc: &WindowContext) {
        update_gamepads(&mut self.gamepads, &wc.settings.gamepad_mappings);

        self.old = self.cur;
        self.read_keyboard(ui);
        self.read_gamepads();
    }

    pub fn held(&self, buttons: u32) -> bool {
        (self.cur & buttons) != 0
    }

    pub fn pressed(&self, buttons: u32) -> bool {
        (self.old & buttons) == 0 && (self.cur & buttons) != 0
    }
}
