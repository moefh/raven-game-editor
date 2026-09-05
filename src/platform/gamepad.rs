use std::collections::HashMap;

pub const GAMEPAD_NONE: u32   = 0;
pub const GAMEPAD_UP: u32     = 1 << 0;
pub const GAMEPAD_DOWN: u32   = 1 << 1;
pub const GAMEPAD_LEFT: u32   = 1 << 2;
pub const GAMEPAD_RIGHT: u32  = 1 << 3;
pub const GAMEPAD_LB: u32     = 1 << 4;
pub const GAMEPAD_RB: u32     = 1 << 5;
pub const GAMEPAD_LT: u32     = 1 << 6;
pub const GAMEPAD_RT: u32     = 1 << 7;
pub const GAMEPAD_L3: u32     = 1 << 8;
pub const GAMEPAD_R3: u32     = 1 << 9;
pub const GAMEPAD_SELECT: u32 = 1 << 10;
pub const GAMEPAD_START: u32  = 1 << 11;
pub const GAMEPAD_HOME: u32   = 1 << 12;

// Nintendo button names:
pub const GAMEPAD_SNES_X: u32 = 1 << 13;  // XBOX_Y, PS_TRIANGLE
pub const GAMEPAD_SNES_Y: u32 = 1 << 14;  // XBOX_X, PS_SQUARE
pub const GAMEPAD_SNES_A: u32 = 1 << 15;  // XBOX_B, PS_CIRCLE
pub const GAMEPAD_SNES_B: u32 = 1 << 16;  // XBOX_A, PS_X

// Playstation button names:
pub const GAMEPAD_PS_TRIANGLE: u32 = GAMEPAD_SNES_X;
pub const GAMEPAD_PS_SQUARE: u32   = GAMEPAD_SNES_Y;
pub const GAMEPAD_PS_CIRCLE: u32   = GAMEPAD_SNES_A;
pub const GAMEPAD_PS_X: u32        = GAMEPAD_SNES_B;

// XBOX button names:
pub const GAMEPAD_XBOX_Y: u32 = GAMEPAD_SNES_X;
pub const GAMEPAD_XBOX_X: u32 = GAMEPAD_SNES_Y;

pub const GAMEPAD_XBOX_B: u32 = GAMEPAD_SNES_A;
pub const GAMEPAD_XBOX_A: u32 = GAMEPAD_SNES_B;

#[derive(Debug)]
pub struct GamepadAxisMapping {
    pub min: u32,
    pub max: u32,
}

impl GamepadAxisMapping {
    pub fn new(min: u32, max: u32) -> Self {
        GamepadAxisMapping {
            min,
            max,
        }
    }
}

#[derive(Debug)]
pub struct GamepadMapping {
    pub buttons: HashMap<u32, u32>,              // buttons[raw] = GAMEPAD_xxx
    pub axes: HashMap<u32, GamepadAxisMapping>,  // axes[raw].min = GAMEPAD_xxx, axes[raw].max = GAMEPAD_xxx
}

impl GamepadMapping {
    pub fn new(buttons: HashMap<u32, u32>, axes: HashMap<u32, GamepadAxisMapping>) -> Self {
        GamepadMapping {
            buttons,
            axes,
        }
    }
}

#[allow(unused)]
pub struct Gamepad {
    pub id: String,
    pub cur: u32,
    pub old: u32,
}

impl Gamepad {
    #[allow(unused)]
    pub fn new(id: String) -> Self {
        Gamepad {
            id,
            cur: 0,
            old: 0,
        }
    }

    #[allow(unused)]
    pub fn held(&self, buttons: u32) -> bool {
        (self.cur & buttons) != 0
    }

    #[allow(unused)]
    pub fn pressed(&self, buttons: u32) -> bool {
        (self.old & buttons) == 0 && (self.cur & buttons) != 0
    }
}
