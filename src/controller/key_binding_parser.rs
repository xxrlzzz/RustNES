use std::collections::HashMap;

#[cfg(feature = "use_gl")]
pub type KeyType = glfw::Key;

#[cfg(feature = "use_sdl2")]
pub type KeyType = sdl2::keyboard::Keycode;

#[cfg(target_arch = "wasm32")]
pub type KeyType = u32;

pub const TOTAL_BUTTONS: usize = 8;
const BUTTONS: &'static [&str] = &["a", "b", "select", "start", "up", "down", "left", "right"];
const KEYBOARD_KEYS: &'static [&str] = &[
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
  "Num0",
  "Num1",
  "Num2",
  "Num3",
  "Num4",
  "Num5",
  "Num6",
  "Num7",
  "Num8",
  "Num9",
  "Escape",
  "LControl",
  "LShift",
  "LAlt",
  "LSystem",
  "RControl",
  "RShift",
  "RAlt",
  "RSystem",
  "Menu",
  "LBracket",
  "RBracket",
  "SemiColon",
  "Comma",
  "Period",
  "Quote",
  "Slash",
  "BackSlash",
  "Tilde",
  "Equal",
  "Dash",
  "Space",
  "Return",
  "BackSpace",
  "Tab",
  "PageUp",
  "PageDown",
  "End",
  "Home",
  "Insert",
  "Delete",
  "Add",
  "Subtract",
  "Multiply",
  "Divide",
  "Left",
  "Right",
  "Up",
  "Down",
  "Numpad0",
  "Numpad1",
  "Numpad2",
  "Numpad3",
  "Numpad4",
  "Numpad5",
  "Numpad6",
  "Numpad7",
  "Numpad8",
  "Numpad9",
  "F1",
  "F2",
  "F3",
  "F4",
  "F5",
  "F6",
  "F7",
  "F8",
  "F9",
  "F10",
  "F11",
  "F12",
  "F13",
  "F14",
  "F15",
  "Pause",
];

#[cfg(feature = "use_gl")]
const KEYS: &'static [KeyType] = &[
  KeyType::A,
  KeyType::B,
  KeyType::C,
  KeyType::D,
  KeyType::E,
  KeyType::F,
  KeyType::G,
  KeyType::H,
  KeyType::I,
  KeyType::J,
  KeyType::K,
  KeyType::L,
  KeyType::M,
  KeyType::N,
  KeyType::O,
  KeyType::P,
  KeyType::Q,
  KeyType::R,
  KeyType::S,
  KeyType::T,
  KeyType::U,
  KeyType::V,
  KeyType::W,
  KeyType::X,
  KeyType::Y,
  KeyType::Z,
  KeyType::Num0,
  KeyType::Num1,
  KeyType::Num2,
  KeyType::Num3,
  KeyType::Num4,
  KeyType::Num5,
  KeyType::Num6,
  KeyType::Num7,
  KeyType::Num8,
  KeyType::Num9,
  KeyType::Escape,
  KeyType::LeftControl,
  KeyType::LeftShift,
  KeyType::LeftAlt,
  KeyType::LeftSuper,
  KeyType::RightControl,
  KeyType::RightShift,
  KeyType::RightAlt,
  KeyType::RightSuper,
  KeyType::Menu,
  KeyType::LeftBracket,
  KeyType::RightBracket,
  KeyType::Semicolon,
  KeyType::Comma,
  KeyType::Period,
  KeyType::Apostrophe, // `
  KeyType::Slash,
  KeyType::Backslash,
  KeyType::Apostrophe, //TILDE ~
  KeyType::Equal,
  KeyType::Minus,
  KeyType::Space,
  KeyType::Enter,
  KeyType::Backspace,
  KeyType::Tab,
  KeyType::PageUp,
  KeyType::PageDown,
  KeyType::End,
  KeyType::Home,
  KeyType::Insert,
  KeyType::Delete,
  KeyType::KpAdd,
  KeyType::KpSubtract,
  KeyType::KpMultiply,
  KeyType::KpDivide,
  KeyType::Left,
  KeyType::Right,
  KeyType::Up,
  KeyType::Down,
  KeyType::Kp0,
  KeyType::Kp1,
  KeyType::Kp2,
  KeyType::Kp3,
  KeyType::Kp4,
  KeyType::Kp5,
  KeyType::Kp6,
  KeyType::Kp7,
  KeyType::Kp8,
  KeyType::Kp9,
  KeyType::F1,
  KeyType::F2,
  KeyType::F3,
  KeyType::F4,
  KeyType::F5,
  KeyType::F6,
  KeyType::F7,
  KeyType::F8,
  KeyType::F9,
  KeyType::F10,
  KeyType::F11,
  KeyType::F12,
  KeyType::F13,
  KeyType::F14,
  KeyType::F15,
  KeyType::Pause,
];

#[cfg(feature = "use_sdl2")]
const KEYS: &'static [sdl2::keyboard::Keycode] = &[
  KeyType::A,
  KeyType::B,
  KeyType::C,
  KeyType::D,
  KeyType::E,
  KeyType::F,
  KeyType::G,
  KeyType::H,
  KeyType::I,
  KeyType::J,
  KeyType::K,
  KeyType::L,
  KeyType::M,
  KeyType::N,
  KeyType::O,
  KeyType::P,
  KeyType::Q,
  KeyType::R,
  KeyType::S,
  KeyType::T,
  KeyType::U,
  KeyType::V,
  KeyType::W,
  KeyType::X,
  KeyType::Y,
  KeyType::Z,
  KeyType::Num0,
  KeyType::Num1,
  KeyType::Num2,
  KeyType::Num3,
  KeyType::Num4,
  KeyType::Num5,
  KeyType::Num6,
  KeyType::Num7,
  KeyType::Num8,
  KeyType::Num9,
  KeyType::Escape,
  KeyType::LCtrl,
  KeyType::LShift,
  KeyType::LAlt,
  KeyType::LGui,
  KeyType::RCtrl,
  KeyType::RShift,
  KeyType::RAlt,
  KeyType::RGui,
  KeyType::Menu,
  KeyType::LeftBracket,
  KeyType::RightBracket,
  KeyType::Semicolon,
  KeyType::Comma,
  KeyType::Period,
  KeyType::Asterisk, // `
  KeyType::Slash,
  KeyType::Backslash,
  KeyType::Ampersand, //TILDE ~
  KeyType::Equals,
  KeyType::Minus,
  KeyType::Space,
  KeyType::Return,
  KeyType::Backspace,
  KeyType::Tab,
  KeyType::PageUp,
  KeyType::PageDown,
  KeyType::End,
  KeyType::Home,
  KeyType::Insert,
  KeyType::Delete,
  KeyType::KpPlus,
  KeyType::KpMinus,
  KeyType::KpMultiply,
  KeyType::KpDivide,
  KeyType::Left,
  KeyType::Right,
  KeyType::Up,
  KeyType::Down,
  KeyType::Kp0,
  KeyType::Kp1,
  KeyType::Kp2,
  KeyType::Kp3,
  KeyType::Kp4,
  KeyType::Kp5,
  KeyType::Kp6,
  KeyType::Kp7,
  KeyType::Kp8,
  KeyType::Kp9,
  KeyType::F1,
  KeyType::F2,
  KeyType::F3,
  KeyType::F4,
  KeyType::F5,
  KeyType::F6,
  KeyType::F7,
  KeyType::F8,
  KeyType::F9,
  KeyType::F10,
  KeyType::F11,
  KeyType::F12,
  KeyType::F13,
  KeyType::F14,
  KeyType::F15,
  KeyType::Pause,
];

#[cfg(target_arch = "wasm32")]
const KEYS: &'static [u32] = &[];

fn parse_one_player(keys: &HashMap<String, Option<String>>) -> Vec<KeyType> {
  #[cfg(not(feature = "wasm"))]
  let mut res = vec![KeyType::A; TOTAL_BUTTONS + 1];
  #[cfg(feature = "wasm")]
  let mut res = vec![0; TOTAL_BUTTONS + 1];
  for (k, v) in keys {
    if let None = v {
      continue;
    }
    let v = v.as_ref().unwrap();
    let maybe_button = BUTTONS.iter().position(|&e| e == k);
    let maybe_key = KEYBOARD_KEYS.iter().position(|&e| e == v);
    if let Some(button) = maybe_button {
      if let Some(key) = maybe_key {
        res[button] = KEYS[key];
      }
    }
  }
  res
}

pub fn parse_key_binding(file: &str) -> (Vec<KeyType>, Vec<KeyType>) {
  let file = ini!(file);
  (
    parse_one_player(&file["player1"]),
    parse_one_player(&file["player2"]),
  )
}

pub fn default_key_binding() -> (Vec<KeyType>, Vec<KeyType>) {
  let file = include_str!("../../assets/keybindings.ini");
  let content = inistr!(file);
  (
    parse_one_player(&content["player1"]),
    parse_one_player(&content["player2"]),
  )
}

mod test {
  #[test]
  fn parse_test() {
    let (player1, _) =
      crate::controller::key_binding_parser::parse_key_binding("./assets/keybindings.ini");
    for key in player1 {
      print!("{:?} ", key);
    }
  }
}
