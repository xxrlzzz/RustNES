use clap::Parser;

use rust_nes::{controller, emulator, logger};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "assets/mario.nes")]
  rom_path: String,

  #[clap(short, long, default_value = "assets/keybindings.ini")]
  key_binding_path: String,

  #[clap(short, long, default_value = "2.0")]
  scale: f32,

  #[clap(long, default_value = "assets/save/saved.json")]
  save_path: String,
}

fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let args = Args::parse();
  let (p1_key, p2_key) = controller::key_binding_parser::parse_key_binding(&args.key_binding_path);
  let mut emulator = emulator::Emulator::new(args.scale, args.save_path, p1_key, p2_key);

  emulator.run(&args.rom_path);
}
