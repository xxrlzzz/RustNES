use std::{collections::HashSet, sync::Mutex};

use log::info;
use wasm_bindgen::{prelude::*, JsCast};

use super::{
  key_binding_parser::{KeyType, TOTAL_BUTTONS},
  Controller,
};

lazy_static! {
  pub static ref KEYBOARD_STATE: Mutex<HashSet<KeyType>> = Mutex::new(HashSet::<KeyType>::new());
}

fn window() -> web_sys::Window {
  web_sys::window().expect("no global `window` exists")
}

pub fn keyboard_listen() -> Result<(), JsValue> {
  info!("keyboard_listen");
  {
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
      // message_keydown.set_text_content(Some(&format!("keydown: {}", event.key_code())));
      KEYBOARD_STATE
        .lock()
        .unwrap()
        .insert(event.key_code() as usize);
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();
  }
  {
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
      // message_keyup.set_text_content(Some(&format!("keyup: {}", event.key_code())));
      KEYBOARD_STATE
        .lock()
        .unwrap()
        .remove(&(event.key_code() as usize));
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
    closure.forget();
  }
  Ok(())
}

impl Controller {
  pub(crate) fn update_keys(&mut self) {
    let keyboard_state = KEYBOARD_STATE.lock().unwrap();
    for button in 0..TOTAL_BUTTONS {
      let pressed = keyboard_state.contains(&self.key_bindings[button]);
      self.key_states |= (pressed as u8) << button;
    }
  }

  pub(crate) fn read_key(&self, btn: &KeyType) -> bool {
    let keyboard_state = KEYBOARD_STATE.lock().unwrap();
    keyboard_state.contains(btn)
  }
}
