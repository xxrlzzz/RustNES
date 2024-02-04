use clap::Parser;
use rust_emu_common::{logger, controller, emulator};
use rust_emu_gba::instance::init_rom_from_path;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "../assets/Tetris(World).gb")]
  rom_path: String,
  // relative path to where you run cmd.
  #[clap(short, long, default_value = "../assets/keybindings.ini")]
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
  let instance = init_rom_from_path(&args.rom_path, &emulator.runtime_config).expect("Failed to load rom.");
  emulator.run(instance);
}

#[cfg(not(any(feature = "use_gl", feature = "use_sdl2")))]
fn main() {
  println!("Please use feature `use_gl` or `use_sdl2` to run this program.");
}
