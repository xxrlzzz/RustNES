
use std::sync::Mutex;

use wasm_bindgen::prelude::*;
use super::Controller;
use super::key_binding_parser::{KeyType, TOTAL_BUTTONS};
use super::web_key::{KEYBOARD_EVENTS, WebEvent};

lazy_static! {
    pub static ref KEYBOARD_STATE: Mutex<u32> = Mutex::new(0);
}
  

#[wasm_bindgen] 
pub fn set_event_key(key_code: u32) {
    KEYBOARD_EVENTS.lock().unwrap().push(WebEvent::KeyDown(key_code as KeyType));
}

#[wasm_bindgen] 
pub fn set_key_down(key_code: u32) {
    let mut val = KEYBOARD_STATE.lock().unwrap();
    *val = *val | key_code;
}

#[wasm_bindgen] 
pub fn set_key_up(key_code: u32) {
    let mut val = KEYBOARD_STATE.lock().unwrap();
    *val = *val &!key_code;
}

impl Controller {
    pub(crate) fn update_keys(&mut self) {
        let val = KEYBOARD_STATE.lock().unwrap();
        self.key_states = (*val) as u8;
    }

    pub(crate) fn read_key(&self, btn: &KeyType) -> bool {
        // TODO
        // log::info!("read_key: {}", btn);
        return false
    }
}