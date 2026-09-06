use std::io::{Result, Error};
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

use wasm_bindgen::JsCast;
use super::gamepad;
use super::gamepad_buttons::{*};
use super::KeyboardPressed;

#[allow(unused)]
pub fn console_log(msg: impl AsRef<str>) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg.as_ref()));
}

fn get_local_storage() -> Result<web_sys::Storage> {
    let window = web_sys::window().ok_or_else(|| Error::other("can't access browser window"))?;
    window
        .local_storage()
        .map_err(|e| { Error::other(format!("{:?}", e)) })?
        .ok_or_else(|| Error::other("can't access brower local storage"))
}

pub fn write_settings_file(filename: impl AsRef<str>, content: &str) -> Result<()> {
    get_local_storage()?
        .set_item(filename.as_ref(), content)
        .map_err(|e| { Error::other(format!("{:?}", e)) })
}

pub fn read_settings_file(filename: impl AsRef<str>) -> Result<String> {
    get_local_storage()?
        .get_item(filename.as_ref())
        .map_err(|e| { Error::other(format!("{:?}", e)) })?
        .ok_or_else(|| Error::other("settings file not found"))
}

pub fn current_time_as_millis() -> u64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|perf| perf.now())
        .map(|now| now.round().abs())
        .unwrap_or(0.0) as u64
}

pub fn current_time_as_string() -> String {
    "<unknown time>".to_owned()
}

pub fn get_event_key(event: &egui::Event) -> Option<KeyboardPressed> {
    match event {
        egui::Event::Copy => {
            Some(KeyboardPressed::CommandC)
        }

        egui::Event::Cut => {
            Some(KeyboardPressed::CommandX)
        }

        // we handle paste by listening for Command+V because Event::Paste
        // is not generated when there's nothing to paste
        egui::Event::Key { key: egui::Key::V, pressed: true, modifiers: egui::Modifiers { command: true, .. }, .. } => {
            Some(KeyboardPressed::CommandV)
        }

        _ => {
            //console_log(format!("{:?}", event));
            None
        }
    }
}

pub fn setup_confirmation_on_close(editor_is_dirty: Arc<Mutex<bool>>) {
    use wasm_bindgen::prelude::*;

    let window = match web_sys::window() {
        Some(window) => { window }
        None => {
            console_log(format!("WARNING: main window not found while setting up close confirmation"));
            return;
        }
    };
    let closure = Closure::wrap(Box::new(move |event: web_sys::BeforeUnloadEvent| {
        let is_dirty = {
            *editor_is_dirty.lock().unwrap()
        };
        if is_dirty {
            event.set_return_value("Closing the window will discard the changes since last save.");
        }
    }) as Box<dyn FnMut(web_sys::BeforeUnloadEvent)>);

    if let Err(e) = window.add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref()) {
        console_log(format!("WARNING: failed to add 'beforeunload' listener while setting up close confirmation: {:?}", e));
        return;
    }

    closure.forget();
}

#[derive(Clone, Copy, Debug)]
enum GamepadRawEventType {
    None,
    Button(u32),
    MinAxis(u32),
    MaxAxis(u32),
}

impl GamepadRawEventType {
    fn button(button: u32) -> Self {
        GamepadRawEventType::Button(button)
    }

    fn axis(axis: u32, val: f64) -> Self {
        if val <= -0.9 {
            GamepadRawEventType::MinAxis(axis)
        } else if val >= 0.9 {
            GamepadRawEventType::MaxAxis(axis)
        } else {
            GamepadRawEventType::None
        }
    }

    fn is_button(self, btn: u32) -> bool {
        if let GamepadRawEventType::Button(b) = self && b == btn { true } else { false }
    }

    fn is_min_axis(self, axis: u32) -> bool {
        if let GamepadRawEventType::MinAxis(a) = self && a == axis { true } else { false }
    }

    fn is_max_axis(self, axis: u32) -> bool {
        if let GamepadRawEventType::MinAxis(a) = self && a == axis { true } else { false }
    }

    fn is_axis(self, axis: u32, val: f64) -> bool {
        if let GamepadRawEventType::MinAxis(a) = self && a == axis && val <= -0.9 { return true; }
        if let GamepadRawEventType::MaxAxis(a) = self && a == axis && val >=  0.9 { return true; }
        false
    }
}

pub struct GamepadManager {
    gamepads: Vec<gamepad::Gamepad>,
    active_gamepad_index: usize,
    raw_events: Vec<gamepad::RawEvent>,
    last_raw_event: Vec<GamepadRawEventType>,
}

impl GamepadManager {
    pub fn new() -> Self {
        GamepadManager {
            gamepads: Vec::new(),
            active_gamepad_index: 0,
            raw_events: Vec::new(),
            last_raw_event: Vec::new(),
        }
    }

    pub fn gamepads(&self) -> impl Iterator<Item = &gamepad::Gamepad> {
        self.gamepads.iter()
    }

    pub fn active_gamepad(&self) -> Option<&gamepad::Gamepad> {
        self.gamepads.get(self.active_gamepad_index)
    }

    fn read_standard_gamepad(gp: &web_sys::Gamepad) -> std::result::Result<u32, wasm_bindgen::JsValue> {
        let mut flags = 0;
        let buttons = gp.buttons();
        for btn in 0..buttons.length() {
            let button: web_sys::GamepadButton = buttons.get(btn).dyn_into()?;
            if button.pressed() {
                flags |= match btn {
                    0 => { GAMEPAD_SNES_B }
                    1 => { GAMEPAD_SNES_A }
                    2 => { GAMEPAD_SNES_Y }
                    3 => { GAMEPAD_SNES_X }
                    4 => { GAMEPAD_LB }
                    5 => { GAMEPAD_RB }
                    6 => { GAMEPAD_LT }
                    7 => { GAMEPAD_RT }
                    8 => { GAMEPAD_SELECT }
                    9 => { GAMEPAD_START }
                    10 => { GAMEPAD_L3 }
                    11 => { GAMEPAD_R3 }
                    12 => { GAMEPAD_UP }
                    13 => { GAMEPAD_DOWN }
                    14 => { GAMEPAD_LEFT }
                    15 => { GAMEPAD_RIGHT }
                    16 => { GAMEPAD_HOME }
                    _ => { 0 }
                }
            }
        }
        Ok(flags)
    }

    fn read_mapped_gamepad(gp: &web_sys::Gamepad, map: &gamepad::Mapping) -> std::result::Result<u32, wasm_bindgen::JsValue> {
        let mut flags = 0;

        let buttons = gp.buttons();
        for button_index in 0..buttons.length().min(map.buttons.len() as u32) {
            let button: web_sys::GamepadButton = buttons.get(button_index).dyn_into()?;
            if button.pressed() {
                flags |= map.buttons.get(&button_index).unwrap_or(&0);
            }
        }

        let axes = gp.axes();
        for axis_index in 0..axes.length().min(map.axes.len() as u32) {
            if let Some(val) = axes.get(axis_index).as_f64() {
                if val <= -0.9 { flags |= map.axes.get(&axis_index).map(|axis| axis.min).unwrap_or(0); }
                if val >=  0.9 { flags |= map.axes.get(&axis_index).map(|axis| axis.max).unwrap_or(0); }
            }
        }
        Ok(flags)
    }

    pub fn get_raw_events(&mut self) -> &[gamepad::RawEvent] {
        fn get_raw_events(
            gamepads: &mut Vec<gamepad::Gamepad>,
            raw_events: &mut Vec<gamepad::RawEvent>,
            last_raw_events: &mut Vec<GamepadRawEventType>,
            active_gamepad_index: &mut usize
        ) -> std::result::Result<(), wasm_bindgen::JsValue> {
            let window = web_sys::window().ok_or(wasm_bindgen::JsValue::from_str("can't find browser window"))?;
            let gp_array = window.navigator().get_gamepads()?;

            gamepads.resize_with(gp_array.length() as usize, || gamepad::Gamepad::new(String::new()));
            last_raw_events.resize_with(gp_array.length() as usize, || GamepadRawEventType::None);

            for (index, ((gp, gamepad), last)) in gp_array.iter().zip(gamepads.iter_mut()).zip(last_raw_events.iter_mut()).enumerate() {
                if gp.is_null() || gp.is_undefined() { continue; }
                let gp: web_sys::Gamepad = gp.dyn_into()?;
                if ! gp.connected() { continue; }
                *active_gamepad_index = index;

                let gp_id = gp.id();
                if gp_id != gamepad.id {
                    gamepad.id.replace_range(.., &gp_id);
                    console_log(format!("DETECTED GAMEPAD: {}", &gamepad.id));
                }

                let buttons = gp.buttons();
                for btn in 0..buttons.length() {
                    let button: web_sys::GamepadButton = buttons.get(btn).dyn_into()?;
                    if button.pressed() {
                        if ! last.is_button(btn) {
                            console_log(format!("pressed button {}", btn));
                            *last = GamepadRawEventType::button(btn);
                            raw_events.push(gamepad::RawEvent::Button { code: btn })
                        }
                    } else if last.is_button(btn) {
                        console_log(format!("released button {}", btn));
                        *last = GamepadRawEventType::None;
                    }
                }

                let axes = gp.axes();
                for axis in 0..axes.length() {
                    if let Some(val) = axes.get(axis).as_f64() {
                        if val < -1.1 || val > 1.1 {
                            console_log(format!("ignoring invalid val={} for axis {}", val, axis));
                            continue;
                        }
                        if val <= -0.9 || val >= 0.9 {
                            if ! last.is_axis(axis, val) {
                                console_log(format!("pressed axis {} with val {}", axis, val));
                                *last = GamepadRawEventType::axis(axis, val);
                                raw_events.push(gamepad::RawEvent::Axis { code: axis, val: val as f32 });
                            }
                        } else if last.is_min_axis(axis) || last.is_max_axis(axis) {
                            console_log(format!("released axis {}", axis));
                            *last = GamepadRawEventType::None;
                        }
                    }
                }
            }
            Ok(())
        }

        self.raw_events.clear();
        if let Err(e) = get_raw_events(&mut self.gamepads, &mut self.raw_events, &mut self.last_raw_event, &mut self.active_gamepad_index) {
            web_sys::console::log_2(&wasm_bindgen::JsValue::from_str("ERROR reading gamepads:"), &e);
        }
        &self.raw_events
    }

    pub fn update(&mut self, mappings: &BTreeMap<String, gamepad::Mapping>) -> bool {
        fn read_gamepads(
            gamepads: &mut Vec<gamepad::Gamepad>,
            mappings: &BTreeMap<String, gamepad::Mapping>,
            active_gamepad_index: &mut usize,
        ) -> std::result::Result<(), wasm_bindgen::JsValue> {
            let window = web_sys::window().ok_or(wasm_bindgen::JsValue::from_str("can't find browser window"))?;
            let gp_array = window.navigator().get_gamepads()?;

            gamepads.resize_with(gp_array.length() as usize, || gamepad::Gamepad::new(String::new()));

            for (index, (gp, gamepad)) in gp_array.iter().zip(gamepads.iter_mut()).enumerate() {
                if gp.is_null() || gp.is_undefined() { continue; }
                let gp: web_sys::Gamepad = gp.dyn_into()?;
                if ! gp.connected() { continue; }

                *active_gamepad_index = index;
                gamepad.cur = match gp.mapping() {
                    web_sys::GamepadMappingType::Standard => {
                        GamepadManager::read_standard_gamepad(&gp)?
                    }
                    _ => {
                        // non-standard mapping: read id and use the corresponding mapping, if any
                        let gp_id = gp.id();
                        if gp_id != gamepad.id {
                            gamepad.id.replace_range(.., &gp_id);
                            gamepad.old = 0;
                            console_log(format!("DETECTED GAMEPAD: {}", &gamepad.id));
                        }
                        if let Some(mapping) = mappings.get(&gp_id) {
                            GamepadManager::read_mapped_gamepad(&gp, mapping)?
                        } else {
                            GamepadManager::read_standard_gamepad(&gp)?  // no mapping defined: try with standard anyway
                        }
                    }
                };
            }
            Ok(())
        }

        for gamepad in self.gamepads.iter_mut() {
            gamepad.old = gamepad.cur;
        }
        match read_gamepads(&mut self.gamepads, mappings, &mut self.active_gamepad_index) {
            Ok(_) => {
                true
            }
            Err(e) => {
                web_sys::console::log_2(&wasm_bindgen::JsValue::from_str("ERROR reading gamepads:"), &e);
                false
            }
        }
    }
}
