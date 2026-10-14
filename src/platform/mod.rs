pub mod gamepad;
pub mod gamepad_buttons;
mod sound;

pub use sound::SoundPlayer;

pub enum KeyboardPressed {
    CommandC,
    CommandV,
    CommandX,
}

cfg_select! {
    target_arch = "wasm32" => {
        mod wasm;
        pub use wasm::{*};
    }

    _ => {
        mod native;
        pub use native::{*};
    }
}
