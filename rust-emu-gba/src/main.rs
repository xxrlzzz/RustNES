use rust_emu_common::logger;

fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  println!("hahaha")
}
