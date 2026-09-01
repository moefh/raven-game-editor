pub const GAMEPAD_NONE: u32   = 0;
pub const GAMEPAD_UP: u32     = 1 << 0;
pub const GAMEPAD_DOWN: u32   = 1 << 1;
pub const GAMEPAD_LEFT: u32   = 1 << 2;
pub const GAMEPAD_RIGHT: u32  = 1 << 3;
pub const GAMEPAD_X: u32      = 1 << 4;
pub const GAMEPAD_Y: u32      = 1 << 5;
pub const GAMEPAD_A: u32      = 1 << 6;
pub const GAMEPAD_B: u32      = 1 << 7;
pub const GAMEPAD_LB: u32     = 1 << 8;
pub const GAMEPAD_RB: u32     = 1 << 9;
pub const GAMEPAD_LT: u32     = 1 << 10;
pub const GAMEPAD_RT: u32     = 1 << 11;
pub const GAMEPAD_L3: u32     = 1 << 12;
pub const GAMEPAD_R3: u32     = 1 << 13;
pub const GAMEPAD_SELECT: u32 = 1 << 14;
pub const GAMEPAD_START: u32  = 1 << 15;
pub const GAMEPAD_HOME: u32   = 1 << 16;

#[derive(Debug)]
pub struct GamepadAxisMapping {
    pub min: u32,
    pub max: u32,
}

impl GamepadAxisMapping {
    pub const EMPTY: Self = GamepadAxisMapping { min: 0, max: 0 };

    pub fn new(min: u32, max: u32) -> Self {
        GamepadAxisMapping {
            min,
            max,
        }
    }
}

#[derive(Debug)]
pub struct GamepadMapping {
    pub buttons: [u32; 32],              // buttons[web] = GAMEPAD_...
    pub axes: [GamepadAxisMapping; 16],  // axes[web].min = GAMEPAD_..., axes[web].max = GAMEPAD_...
}

impl GamepadMapping {
    pub const BUTTON_MAPPING_EMPTY: u32 = 0;

    pub fn new() -> Self {
        GamepadMapping {
            buttons: [Self::BUTTON_MAPPING_EMPTY; 32],
            axes: [GamepadAxisMapping::EMPTY; 16],
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
