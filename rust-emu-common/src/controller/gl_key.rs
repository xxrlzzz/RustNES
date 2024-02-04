use super::{
  key_binding_parser::{KeyType, TOTAL_BUTTONS},
  Controller,
};

pub static mut WINDOW_INSTANCE: Option<std::rc::Rc<std::cell::RefCell<glfw::Window>>> = None;

impl Controller {
  pub fn update_keys(&mut self) {
    let window_ref = unsafe { WINDOW_INSTANCE.as_ref().unwrap().borrow() };
    for button in 0..TOTAL_BUTTONS {
      let pressed = window_ref.get_key(self.key_bindings[button]) == glfw::Action::Press;
      self.key_states |= (pressed as u8) << button;
    }
  }

  pub fn read_key(&self, btn: &KeyType) -> bool {
    let window_ref = unsafe { WINDOW_INSTANCE.as_ref().unwrap().borrow() };
    window_ref.get_key(*btn) == glfw::Action::Press
  }
}
