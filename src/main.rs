mod lib;
use lib::*;
#[macro_use]
extern crate ini;

fn main() {
  match crate::logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let mut emulator = lib::emulator::Emulator::new();
  let (p1_key, p2_key) =
    controller::key_binding_parser::parse_key_binding("assets/keybindings.ini");
  emulator.set_keys(p1_key, p2_key);
  emulator.run("assets/test.nes");
}
