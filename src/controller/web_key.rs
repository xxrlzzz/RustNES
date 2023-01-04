use std::{collections::HashSet, sync::Mutex};

use js_sys::Array;
use log::info;
use wasm_bindgen::{prelude::*, JsCast};

use super::{
  key_binding_parser::{KeyType, TOTAL_BUTTONS},
  Controller,
};

pub enum WebEvent {
  KeyDown(KeyType),
  Focus(bool),
}
lazy_static! {
  pub static ref KEYBOARD_STATE: Mutex<HashSet<KeyType>> = Mutex::new(HashSet::<KeyType>::new());
  pub static ref KEYBOARD_EVENTS: Mutex<Vec<WebEvent>> = Mutex::new(Vec::<WebEvent>::new());
  pub static ref REMOTE_KEYBOARD_STATE: Mutex<HashSet<KeyType>> =
    Mutex::new(HashSet::<KeyType>::new());
}

#[wasm_bindgen]
pub fn keyboard_status() -> Array {
  let mut keyboard_vec = Vec::<i32>::new();
  {
    let keyboard_state = KEYBOARD_STATE.lock().unwrap();
    for key in keyboard_state.iter() {
      keyboard_vec.push(*key as i32);
    }
    // log::info!("keyboard_state size : {:?}", keyboard_state.len());
  }
  keyboard_vec.into_iter().map(JsValue::from).collect()
}

#[wasm_bindgen]
pub fn update_remote_keyboard_state(keyboard_state: Array) {
  let mut remote_keyboard_state = REMOTE_KEYBOARD_STATE.lock().unwrap();
  remote_keyboard_state.clear();
  for key in keyboard_state.iter() {
    remote_keyboard_state.insert(key.as_f64().unwrap() as KeyType);
  }
}

fn window() -> web_sys::Window {
  web_sys::window().expect("no global `window` exists")
}

pub fn keyboard_listen() -> Result<(), JsValue> {
  info!("keyboard_listen");
  {
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
      KEYBOARD_STATE
        .lock()
        .unwrap()
        .insert(event.key_code() as usize);
      KEYBOARD_EVENTS
        .lock()
        .unwrap()
        .push(WebEvent::KeyDown(event.key_code() as KeyType));
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();
  }
  {
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
      KEYBOARD_STATE
        .lock()
        .unwrap()
        .remove(&(event.key_code() as usize));
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
    closure.forget();
  }

  {
    let closure = Closure::wrap(Box::new(move |_: web_sys::FocusEvent| {
      KEYBOARD_EVENTS.lock().unwrap().push(WebEvent::Focus(true));
    }) as Box<dyn FnMut(_)>);
    window().set_onfocus(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
  }
  {
    let closure = Closure::wrap(Box::new(move |_: web_sys::FocusEvent| {
      KEYBOARD_EVENTS.lock().unwrap().push(WebEvent::Focus(false));
    }) as Box<dyn FnMut(_)>);
    window().set_onblur(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
  }
  Ok(())
}

impl Controller {
  pub(crate) fn update_keys(&mut self) {
    let keyboard_state = if self.enable_remote {
      let mut state = REMOTE_KEYBOARD_STATE.lock().unwrap();
      let mut result = HashSet::new();
      std::mem::swap(&mut result, &mut *state);
      result
    } else {
      KEYBOARD_STATE.lock().unwrap().clone()
    };
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

pub fn pull_events() -> Vec<WebEvent> {
  let mut events = KEYBOARD_EVENTS.lock().unwrap();
  let mut result = Vec::new();
  std::mem::swap(&mut result, &mut *events);
  result
}
