use std::io::{Result, Error};

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
        .unwrap()
        .performance()
        .unwrap()
        .now()
        .round() as u64
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
        // is only generated when there's anything to paste:
        //egui::Event::Paste(_) => {
        //    Some(KeyboardPressed::CommandV)
        //}

        egui::Event::Key { key: egui::Key::V, pressed: true, modifiers: egui::Modifiers { command: true, .. }, .. } => {
            Some(KeyboardPressed::CommandV)
        }

        _ => {
            //console_log(format!("{:?}", event));
            None
        }
    }
}
