#[cfg(feature = "use_gl")]
use std::cell::RefCell;
#[cfg(feature = "use_gl")]
use std::rc::Rc;

use crate::common::bit_eq;
use crate::common::types::Byte;

pub mod key_binding_parser;

use key_binding_parser::TOTAL_BUTTONS;

use self::key_binding_parser::KeyType;

#[derive(Default)]
pub struct Controller {
  enable_strobe: bool,
  key_states: u8,
  key_bindings: Vec<KeyType>,
  #[cfg(feature = "use_gl")]
  window: Option<Rc<RefCell<glfw::Window>>>,
}

impl Controller {
  pub fn new() -> Self {
    Self {
      enable_strobe: false,
      key_states: 0,
      key_bindings: vec![KeyType::A; TOTAL_BUTTONS],
      #[cfg(feature = "use_gl")]
      window: None,
    }
  }

  #[cfg(feature = "use_gl")]
  pub fn set_window(&mut self, window: Rc<RefCell<glfw::Window>>) {
    self.window = Some(window);
  }

  pub fn set_key_bindings(&mut self, keys: Vec<KeyType>) {
    self.key_bindings = keys;
  }

  pub fn strobe(&mut self, b: Byte) {
    self.enable_strobe = bit_eq(b, 1);
    if !self.enable_strobe {
      self.key_states = 0;

      #[cfg(feature = "use_sfml")]
      for button in 0..TOTAL_BUTTONS {
        let offset = (self.key_bindings[button].is_pressed() as u8) << button;
        self.key_states |= offset;
      }

      #[cfg(feature = "use_gl")]
      if self.window.is_some() {
        let window_ref = self.window.as_ref().unwrap().borrow();
        for button in 0..TOTAL_BUTTONS {
          let pressed = window_ref.get_key(self.key_bindings[button]) == glfw::Action::Press;
          self.key_states |= (pressed as u8) << button;
        }
      }
    }
  }

  pub fn read(&mut self) -> Byte {
    return if self.enable_strobe {
      #[cfg(feature = "use_sfml")]
      return self.key_bindings[0].is_pressed() as u8 | 0x40;

      #[cfg(feature = "use_gl")]
      return (self
        .window
        .as_ref()
        .unwrap()
        .borrow()
        .get_key(self.key_bindings[0])
        == glfw::Action::Press) as u8
        | 0x40;
    } else {
      let ret = self.key_states & 1;
      self.key_states >>= 1;
      ret
    } | 0x40;
  }
}
