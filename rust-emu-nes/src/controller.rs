use rust_emu_common::{
  controller::{key_binding_parser::KeyType, Controller},
  types::*,
};

#[derive(Default)]
pub(crate) struct NESController{ 
  inner:Controller, 
  enable_strobe: bool,
}

impl NESController {
  pub fn new() -> Self {
    Self {
      inner: Controller::new(),
      enable_strobe: false,
    }
  }

  pub fn remote_controller() -> Self {
    Self {
      inner: Controller::remote_controller(),
      enable_strobe:false,
    }
  }

  pub fn strobe(&mut self, b: Byte) {
    self.enable_strobe = bit_eq(b, 1);
    if !self.enable_strobe {
      self.inner.key_states = 0;
      self.inner.update_keys();
    }
  }

  pub fn read(&mut self) -> Byte {
    return if self.enable_strobe {
      self.inner.read_key(&self.inner.key_bindings[0]) as u8 | 0x40
    } else {
      let ret = self.inner.key_states & 1;
      self.inner.key_states >>= 1;
      ret
    } | 0x40;
  }

  pub fn set_key_bindings(&mut self, keys: Vec<KeyType>) {
    self.inner.key_bindings = keys;
  }
}
