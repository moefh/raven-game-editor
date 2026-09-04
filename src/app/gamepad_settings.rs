use std::collections::HashMap;

use crate::platform::gamepad::{
    self,
    GamepadMapping,
    GamepadAxisMapping,
};

pub fn get_default() -> HashMap<String, GamepadMapping> {
    HashMap::from([
        (String::from("Zikway HID gamepad (Vendor: 3537 Product: 1041)"), GamepadMapping {
            buttons: [
                gamepad::GAMEPAD_XBOX_B,
                gamepad::GAMEPAD_XBOX_A,
                0,
                gamepad::GAMEPAD_XBOX_Y,
                gamepad::GAMEPAD_XBOX_X,
                0,
                gamepad::GAMEPAD_LB,
                gamepad::GAMEPAD_RB,
                gamepad::GAMEPAD_LT,
                gamepad::GAMEPAD_RT,
                gamepad::GAMEPAD_SELECT,
                gamepad::GAMEPAD_START,
                gamepad::GAMEPAD_HOME,
                gamepad::GAMEPAD_L3,
                gamepad::GAMEPAD_R3,
                0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            axes: [
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::new(gamepad::GAMEPAD_LEFT, gamepad::GAMEPAD_RIGHT),
                GamepadAxisMapping::new(gamepad::GAMEPAD_UP, gamepad::GAMEPAD_DOWN),
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
            ],
        }),

        (String::from("0079-0006-Microntek              USB Joystick          "), GamepadMapping {
            buttons: [
                gamepad::GAMEPAD_PS_TRIANGLE,
                gamepad::GAMEPAD_PS_CIRCLE,
                gamepad::GAMEPAD_PS_X,
                gamepad::GAMEPAD_PS_SQUARE,
                gamepad::GAMEPAD_LB,
                gamepad::GAMEPAD_RB,
                gamepad::GAMEPAD_LT,
                gamepad::GAMEPAD_RT,
                gamepad::GAMEPAD_SELECT,
                gamepad::GAMEPAD_START,
                gamepad::GAMEPAD_L3,
                gamepad::GAMEPAD_R3,
                0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            axes: [
                GamepadAxisMapping::new(gamepad::GAMEPAD_LEFT, gamepad::GAMEPAD_RIGHT),
                GamepadAxisMapping::new(gamepad::GAMEPAD_UP, gamepad::GAMEPAD_DOWN),
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
                GamepadAxisMapping::EMPTY,
            ],
        }),

    ])
}
