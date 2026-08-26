use std::io::{Result, Error};
use std::sync::{Arc, Mutex};

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
