use std::io::{Result, Error};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::collections::{
    BTreeMap,
    HashMap,
};

use super::gamepad;
use super::gamepad_buttons::{*};
use super::KeyboardPressed;

const APP_ID: &str = "raven-game-editor";

static TIMESTAMP_FORMAT: LazyLock<time::format_description::FormatDescriptionV3> = LazyLock::new(|| {
    time::format_description::parse_borrowed::<3>("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap()
});

pub struct GamepadManager {
    gilrs: Option<gilrs::Gilrs>,
    gamepads: HashMap<gilrs::GamepadId, gamepad::Gamepad>,
    active_gamepad_id: Option<gilrs::GamepadId>,
    raw_events: Vec<gamepad::RawEvent>,
}

impl GamepadManager {
    pub fn new() -> Self {
        GamepadManager {
            gilrs: gilrs::Gilrs::new().ok(),
            gamepads: HashMap::new(),
            active_gamepad_id: None,
            raw_events: Vec::new(),
        }
    }

    pub fn active_gamepad(&self) -> Option<&gamepad::Gamepad> {
        self.active_gamepad_id.and_then(|id| self.gamepads.get(&id))
    }

    pub fn gamepads(&self) -> impl Iterator<Item = &gamepad::Gamepad> {
        self.gamepads.values()
    }

    pub fn get_raw_events(&mut self) -> &[gamepad::RawEvent] {
        self.raw_events.clear();
        if let Some(gilrs) = &mut self.gilrs {
            while let Some(ev) = gilrs.next_event() {
                self.active_gamepad_id = Some(ev.id);
                let gp = gilrs.gamepad(ev.id);
                self.gamepads.entry(ev.id).or_insert_with(|| {
                    gamepad::Gamepad::new(gp.name().to_owned())
                });
                match ev.event {
                    gilrs::EventType::ButtonPressed(_btn, code) => {
                        self.raw_events.push(gamepad::RawEvent::Button { code: code.into_u32() });
                    }
                    gilrs::EventType::AxisChanged(_axis, val, code) => {
                        self.raw_events.push(gamepad::RawEvent::Axis { code: code.into_u32(), val });
                    }
                    _ => {}
                }
            }
        }
        &self.raw_events
    }

    pub fn update(&mut self, mappings: &BTreeMap<String, gamepad::Mapping>) -> bool {
        if let Some(gilrs) = &mut self.gilrs {
            while let Some(ev) = gilrs.next_event() {
                self.active_gamepad_id = Some(ev.id);
                let gp = gilrs.gamepad(ev.id);
                let gamepad = self.gamepads.entry(ev.id).or_insert_with(|| {
                    gamepad::Gamepad::new(gp.name().to_owned())
                });
                match ev.event {
                    gilrs::EventType::ButtonChanged(btn, val, code) => {
                        let gamepad_val = match btn {
                            gilrs::ev::Button::South         => { GAMEPAD_PS_X }
                            gilrs::ev::Button::East          => { GAMEPAD_PS_CIRCLE }
                            gilrs::ev::Button::North         => { GAMEPAD_PS_TRIANGLE }
                            gilrs::ev::Button::West          => { GAMEPAD_PS_SQUARE }
                            gilrs::ev::Button::LeftTrigger   => { GAMEPAD_LB }
                            gilrs::ev::Button::LeftTrigger2  => { GAMEPAD_LT }
                            gilrs::ev::Button::RightTrigger  => { GAMEPAD_RB }
                            gilrs::ev::Button::RightTrigger2 => { GAMEPAD_RT }
                            gilrs::ev::Button::Select        => { GAMEPAD_SELECT }
                            gilrs::ev::Button::Start         => { GAMEPAD_START }
                            gilrs::ev::Button::Mode          => { GAMEPAD_HOME }
                            gilrs::ev::Button::LeftThumb     => { GAMEPAD_L3 }
                            gilrs::ev::Button::RightThumb    => { GAMEPAD_R3 }
                            gilrs::ev::Button::DPadUp        => { GAMEPAD_UP }
                            gilrs::ev::Button::DPadDown      => { GAMEPAD_DOWN }
                            gilrs::ev::Button::DPadLeft      => { GAMEPAD_LEFT }
                            gilrs::ev::Button::DPadRight     => { GAMEPAD_RIGHT }
                            gilrs::ev::Button::C             => { 0 }
                            gilrs::ev::Button::Z             => { 0 }
                            gilrs::ev::Button::Unknown => {
                                mappings.get(gp.name())
                                    .and_then(|map| map.buttons.get(&code.into_u32()))
                                    .copied()
                                    .unwrap_or(0)
                            }
                        };
                        if val < 0.5 {
                            gamepad.cur &= !gamepad_val;
                        } else {
                            gamepad.cur |= gamepad_val;
                        }
                    }
                    gilrs::EventType::AxisChanged(_axis, val, code) => {
                        if let Some(gamepad_axis) = mappings.get(gp.name()).and_then(|map| map.axes.get(&code.into_u32())) {
                            if val <= -0.9 {
                                gamepad.cur |= gamepad_axis.min;
                                gamepad.cur &= !gamepad_axis.max;
                            } else if val >= 0.9 {
                                gamepad.cur |= gamepad_axis.max;
                                gamepad.cur &= !gamepad_axis.min;
                            } else {
                                gamepad.cur &= !(gamepad_axis.min|gamepad_axis.max);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        ! self.gamepads.is_empty()
    }
}

fn get_settings_dir() -> Option<PathBuf> {
    use egui::os::OperatingSystem as OS;
    match OS::from_target_os() {
        OS::Nix => std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::home_dir().map(|p| p.join(".config")))
            .map(|p| { p.join(APP_ID) }),
        OS::Mac => std::env::home_dir().map(|p| {
            p.join("Library").join("Preferences").join(APP_ID)
        }),
        OS::Windows => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(APP_ID)),
        _ => None,
    }
}

pub fn write_settings_file(filename: impl AsRef<str>, content: &str) -> Result<()> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::write(&filename, content)
}

pub fn read_settings_file(filename: impl AsRef<str>) -> Result<String> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::read_to_string(&filename)
}

pub fn current_time_as_millis() -> u64 {
    use std::time::{SystemTime, Duration, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    since_the_epoch.as_millis() as u64
}

pub fn current_time_as_string() -> String {
    if let Ok(now) = time::OffsetDateTime::now_local() && let Ok(timestamp) = now.format(&TIMESTAMP_FORMAT) {
        timestamp
    } else {
        "<unknown time>".to_owned()
    }
}

pub fn get_event_key(event: &egui::Event) -> Option<KeyboardPressed> {
    match event {
        egui::Event::Copy => {
            Some(KeyboardPressed::CommandC)
        }

        egui::Event::Cut => {
            Some(KeyboardPressed::CommandX)
        }

        // we have to handle the key release event (pressed: false)
        // because egui doesn't send a paste event when CMD+V is
        // pressed when there's nothing in the clipboard
        egui::Event::Key { key: egui::Key::V, pressed: false, modifiers: egui::Modifiers { command: true, .. }, .. } => {
            Some(KeyboardPressed::CommandV)
        }

        _ => {
            //println!("{:?}", event);
            None
        }
    }
}
