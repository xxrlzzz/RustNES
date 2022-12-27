use crate::common::{bit_eq, Byte};

pub mod key_binding_parser;

use key_binding_parser::{KeyType, TOTAL_BUTTONS};
#[cfg(feature = "wasm")]
pub mod web_key;

#[cfg(feature = "use_gl")]
pub mod gl_key;

#[cfg(feature = "use_sdl2")]
pub mod sdl2_key;

#[derive(Default)]
pub struct Controller {
  enable_strobe: bool,
  key_states: u8,
  key_bindings: Vec<KeyType>,
  enable_remote: bool,
}

impl Controller {
  pub fn new() -> Self {
    Self {
      enable_strobe: false,
      key_states: 0,
      #[cfg(not(feature = "wasm"))]
      key_bindings: vec![KeyType::A; TOTAL_BUTTONS],
      #[cfg(feature = "wasm")]
      key_bindings: vec![0; TOTAL_BUTTONS],
      enable_remote: false,
    }
  }

  pub fn remote_controller() -> Self {
    Self {
      enable_strobe: false,
      key_states: 0,
      #[cfg(not(feature = "wasm"))]
      key_bindings: vec![KeyType::A; TOTAL_BUTTONS],
      #[cfg(feature = "wasm")]
      key_bindings: vec![0; TOTAL_BUTTONS],
      enable_remote: true,
    }
  }

  pub fn set_key_bindings(&mut self, keys: Vec<KeyType>) {
    self.key_bindings = keys;
  }

  pub fn strobe(&mut self, b: Byte) {
    self.enable_strobe = bit_eq(b, 1);
    if !self.enable_strobe {
      self.key_states = 0;
      self.update_keys();
    }
  }

  pub fn read(&mut self) -> Byte {
    return if self.enable_strobe {
      self.read_key(&self.key_bindings[0]) as u8 | 0x40
    } else {
      let ret = self.key_states & 1;
      self.key_states >>= 1;
      ret
    } | 0x40;
  }
}
