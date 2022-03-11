use sfml::window::Key;

use crate::common::bit_eq;
use crate::common::types::Byte;

pub mod key_binding_parser;

use key_binding_parser::TOTAL_BUTTONS;

#[derive(Default, Clone)]
pub struct Controller {
  enable_strobe: bool,
  key_states: u8,
  key_bindings: Vec<Key>,
}

impl Controller {
  pub fn new() -> Self {
    Self {
      enable_strobe: false,
      key_states: 0,
      key_bindings: vec![Key::A; TOTAL_BUTTONS],
    }
  }
  pub fn set_key_bindings(&mut self, keys: Vec<Key>) {
    self.key_bindings = keys;
  }

  pub fn strobe(&mut self, b: Byte) {
    self.enable_strobe = bit_eq(b, 1);
    if !self.enable_strobe {
      self.key_states = 0;
      for button in 0..TOTAL_BUTTONS {
        let offset = (self.key_bindings[button].is_pressed() as u8) << button;
        self.key_states |= offset;
      }
    }
  }

  pub fn read(&mut self) -> Byte {
    return if self.enable_strobe {
      self.key_bindings[0].is_pressed() as u8
    } else {
      let ret = self.key_states & 1;
      self.key_states >>= 1;
      ret
    } | 0x40;
  }
}
