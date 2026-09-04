use std::io::{Result, Error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use wasm_bindgen::JsCast;
use super::gamepad;
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
            console_log("WARNING: main window not found while setting up close confirmation");
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

fn read_standard_gamepad(gp: &web_sys::Gamepad) -> std::result::Result<u32, wasm_bindgen::JsValue> {
    let mut flags = 0;
    let buttons = gp.buttons();
    for btn in 0..buttons.length() {
        let button: web_sys::GamepadButton = buttons.get(btn).dyn_into()?;
        if button.pressed() {
            flags |= match btn {
                0 => { gamepad::GAMEPAD_SNES_B }
                1 => { gamepad::GAMEPAD_SNES_A }
                2 => { gamepad::GAMEPAD_SNES_Y }
                3 => { gamepad::GAMEPAD_SNES_X }
                4 => { gamepad::GAMEPAD_LB }
                5 => { gamepad::GAMEPAD_RB }
                6 => { gamepad::GAMEPAD_LT }
                7 => { gamepad::GAMEPAD_RT }
                8 => { gamepad::GAMEPAD_SELECT }
                9 => { gamepad::GAMEPAD_START }
                10 => { gamepad::GAMEPAD_L3 }
                11 => { gamepad::GAMEPAD_R3 }
                12 => { gamepad::GAMEPAD_UP }
                13 => { gamepad::GAMEPAD_DOWN }
                14 => { gamepad::GAMEPAD_LEFT }
                15 => { gamepad::GAMEPAD_RIGHT }
                16 => { gamepad::GAMEPAD_HOME }
                _ => { 0 }
            }
        }
    }
    Ok(flags)
}

fn read_mapped_gamepad(gp: &web_sys::Gamepad, map: &gamepad::GamepadMapping) -> std::result::Result<u32, wasm_bindgen::JsValue> {
    let mut flags = 0;

    let buttons = gp.buttons();
    for i in 0..buttons.length().min(map.buttons.len() as u32) {
        let button: web_sys::GamepadButton = buttons.get(i).dyn_into()?;
        if button.pressed() {
            flags |= map.buttons[i as usize];
        }
    }

    let axes = gp.axes();
    for i in 0..axes.length().min(map.axes.len() as u32) {
        if let Some(axis) = axes.get(i).as_f64() {
            if axis < -0.9 { flags |= map.axes[i as usize].min; }
            if axis >  0.9 { flags |= map.axes[i as usize].max; }
        }
    }
    Ok(flags)
}

pub fn update_gamepads(gamepads: &mut Vec<gamepad::Gamepad>, mappings: &HashMap<String, gamepad::GamepadMapping>) -> bool {
    fn read_gamepads(
        gamepads: &mut Vec<gamepad::Gamepad>,
        mappings: &HashMap<String, gamepad::GamepadMapping>
    ) -> std::result::Result<bool, wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or(wasm_bindgen::JsValue::from_str("can't find browser window"))?;
        let gp_array = window.navigator().get_gamepads()?;

        gamepads.resize_with(gp_array.length() as usize, || gamepad::Gamepad::new(String::new()));

        let mut has_some_gamepad = false;
        for (gp, gamepad) in gp_array.iter().zip(gamepads.iter_mut()) {
            if gp.is_null() || gp.is_undefined() { continue; }
            let gp: web_sys::Gamepad = gp.dyn_into()?;
            if ! gp.connected() { continue; }

            has_some_gamepad = true;
            gamepad.cur = match gp.mapping() {
                web_sys::GamepadMappingType::Standard => {
                    read_standard_gamepad(&gp)?
                }
                _ => {
                    // non-standard mapping: read id and use the corresponding mapping, if any
                    let gp_id = gp.id();
                    if gp_id != gamepad.id {
                        gamepad.id.replace_range(.., &gp_id);
                        gamepad.old = 0;
                        console_log(format!("DETECTED CONTROLLER: {}", &gamepad.id));
                    }
                    if let Some(mapping) = mappings.get(&gp_id) {
                        read_mapped_gamepad(&gp, mapping)?
                    } else {
                        read_standard_gamepad(&gp)?  // no mapping defined: try with standard anyway
                    }
                }
            };
        }
        Ok(has_some_gamepad)
    }

    for gamepad in gamepads.iter_mut() {
        gamepad.old = gamepad.cur;
    }
    match read_gamepads(gamepads, mappings) {
        Ok(has_some_gamepad) => { has_some_gamepad }
        Err(e) => {
            web_sys::console::log_2(&wasm_bindgen::JsValue::from_str("ERROR reading gamepads:"), &e);
            false
        }
    }
}
