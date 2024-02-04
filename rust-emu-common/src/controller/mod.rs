pub mod key_binding_parser;

use self::key_binding_parser::{KeyType, TOTAL_BUTTONS};
#[cfg(feature = "wasm")]
pub mod web_key;

#[cfg(feature="wasm-miniapp")]
pub mod virtual_key;

#[cfg(feature = "use_gl")]
pub mod gl_key;

#[cfg(feature = "use_sdl2")]
pub mod sdl2_key;

#[derive(Default)]
pub struct Controller {
  pub key_states: u8,
  pub key_bindings: Vec<KeyType>,
  #[allow(dead_code)]
  pub enable_remote: bool,
}


impl Controller {
  pub fn new() -> Self {
    Self {
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

  // impl by submodule
  // pub fn read(&mut self) -> Byte {}
  // pub fn strobe(&mut self, b: Byte) {}

}