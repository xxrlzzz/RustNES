use clap::Parser;

#[cfg(any(feature = "use_gl", feature = "use_sdl2"))]
use rust_nes::{controller, emulator, logger};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "assets/mario.nes")]
  rom_path: String,
  // relative path to where you run cmd.
  #[clap(short, long, default_value = "assets/keybindings.ini")]
  key_binding_path: String,

  #[clap(short, long, default_value = "2.0")]
  scale: f32,

  #[clap(long, default_value = "save/saved.json")]
  save_path: String,
}

#[cfg(any(feature = "use_gl", feature = "use_sdl2"))]
fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let args = Args::parse();
  let (p1_key, p2_key) = controller::key_binding_parser::parse_key_binding(&args.key_binding_path);
  let mut emulator = emulator::Emulator::new(args.scale, args.save_path, p1_key, p2_key);
  let instance = emulator.create_instance(&args.rom_path);
  emulator.run(instance);
}

#[cfg(not(any(feature = "use_gl", feature = "use_sdl2")))]
fn main() {
  println!("Please use feature `use_gl` or `use_sdl2` to run this program.");
}

#[cfg(test)]
mod tests {
  use std::time::Instant;

  type MatrixType = std::collections::HashMap<&'static str, u128>;
  const CNT: u32 = 1788908;

  #[test]
  fn hashmap_test() {
    let mut matrix = MatrixType::default();
    let category_list = vec!["1", "2", "3"];
    let start = Instant::now();
    for i in 0..CNT {
      matrix
        .entry(category_list[i as usize % 3])
        .and_modify(|e| *e += 1)
        .or_insert(0);
    }
    println!("total cost :{:?}", Instant::now() - start);
  }
}
