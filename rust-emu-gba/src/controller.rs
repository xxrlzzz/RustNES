use rust_emu_common::{
  controller::{key_binding_parser::KeyType, Controller},
  types::*,
};

#[derive(Default)]
pub (crate) struct GBAController{
  inner: Controller,

  select_btn: bool,
  select_dir: bool,
}

impl GBAController {
  pub fn new() -> Self {
    Self {
      inner: Controller::new(),
      select_btn: false,
      select_dir: false,
    }
  }

  // pub fn remote_controller() -> Self {
  //   Self {
  //     inner: Controller::remote_controller(),
  //     select_btn: false,
  //     select_dir: false,
  //   }
  // }

  pub fn set_key_bindings(&mut self, keys: Vec<KeyType>) {
    self.inner.key_bindings = keys;
  }

  pub fn strobe(&mut self, b: Byte) {
    self.select_btn = bit_eq(b, 0x20);
    self.select_dir = bit_eq(b, 0x10);
  }

  pub fn read(&mut self) -> Byte {
    let mut b = 0xCF;
    if !self.select_btn {
      if self.inner.read_key(&self.inner.key_bindings[3]) {
        b &= 0xF7;
      }
      if self.inner.read_key(&self.inner.key_bindings[2]) {
        b &= 0xFB;
      }
      if self.inner.read_key(&self.inner.key_bindings[0]) {
        b &= 0xFE;
      }
      if self.inner.read_key(&self.inner.key_bindings[1]) {
        b &= 0xFD;
      }
    }
    if !self.select_dir {
      if self.inner.read_key(&self.inner.key_bindings[6]) {
        b &= 0xFD;
      }
      if self.inner.read_key(&self.inner.key_bindings[7]) {
        b &= 0xFE;
      }
      if self.inner.read_key(&self.inner.key_bindings[4]) {
        b &= 0xFB;
      }
      if self.inner.read_key(&self.inner.key_bindings[5]) {
        b &= 0xF7;
      }
    }
    // log::info!("read key {:02X}", b);
    b
  }
}