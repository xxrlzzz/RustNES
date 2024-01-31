use super::{
  key_binding_parser::{KeyType, TOTAL_BUTTONS},
  Controller,
};

pub static mut KEYBOARD_STATE: Option<std::collections::HashSet<sdl2::keyboard::Keycode>> = None;
impl Controller {
  pub(crate) fn update_keys(&mut self) {
    let keyboard_state = unsafe { KEYBOARD_STATE.as_ref().unwrap() };
    for button in 0..TOTAL_BUTTONS {
      let pressed = keyboard_state.contains(&self.key_bindings[button]);
      self.key_states |= (pressed as u8) << button;
    }
  }

  pub(crate) fn read_key(&self, _btn: &KeyType) -> bool {
    let keyboard_state = unsafe { KEYBOARD_STATE.as_ref().unwrap() };
    keyboard_state.contains(&self.key_bindings[0])
  }
}
