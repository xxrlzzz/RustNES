use std::collections::HashSet;

use crate::common::bit_eq;
use crate::common::Byte;

pub mod key_binding_parser;

use key_binding_parser::TOTAL_BUTTONS;

use self::key_binding_parser::KeyType;

#[cfg(feature = "use_gl")]
pub static mut WINDOW_INSTANCE: Option<std::rc::Rc<std::cell::RefCell<glfw::Window>>> = None;

#[cfg(feature = "use_sdl2")]
pub static mut KEYBOARD_STATE: Option<HashSet<sdl2::keyboard::Keycode>> = None;

#[derive(Default)]
pub struct Controller {
  enable_strobe: bool,
  key_states: u8,
  key_bindings: Vec<KeyType>,
}

impl Controller {
  pub fn new() -> Self {
    Self {
      enable_strobe: false,
      key_states: 0,
      key_bindings: vec![KeyType::A; TOTAL_BUTTONS],
    }
  }

  pub fn set_key_bindings(&mut self, keys: Vec<KeyType>) {
    self.key_bindings = keys;
  }

  pub fn strobe(&mut self, b: Byte) {
    self.enable_strobe = bit_eq(b, 1);
    if !self.enable_strobe {
      self.key_states = 0;

      #[cfg(feature = "use_gl")]
      {
        let window_ref = unsafe { WINDOW_INSTANCE.as_ref().unwrap().borrow() };
        for button in 0..TOTAL_BUTTONS {
          let pressed = window_ref.get_key(self.key_bindings[button]) == glfw::Action::Press;
          self.key_states |= (pressed as u8) << button;
        }
      }

      #[cfg(feature = "use_sdl2")]
      {
        let keyboard_state = unsafe { KEYBOARD_STATE.as_ref().unwrap() };
        for button in 0..TOTAL_BUTTONS {
          let pressed = keyboard_state.contains(&self.key_bindings[button]);
          self.key_states |= (pressed as u8) << button;
        }
      }
    }
  }

  pub fn read(&mut self) -> Byte {
    return if self.enable_strobe {
      #[cfg(feature = "use_gl")]
      {
        let window_ref = unsafe { WINDOW_INSTANCE.as_ref().unwrap().borrow() };
        return (window_ref.get_key(self.key_bindings[0]) == glfw::Action::Press) as u8 | 0x40;
      }
      #[cfg(feature = "use_sdl2")]
      {
        let keyboard_state = unsafe { KEYBOARD_STATE.as_ref().unwrap() };
        return keyboard_state.contains(&self.key_bindings[0]) as u8 | 0x40;
      }
    } else {
      let ret = self.key_states & 1;
      self.key_states >>= 1;
      ret
    } | 0x40;
  }
}
