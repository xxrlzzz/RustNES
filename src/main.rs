use clap::Parser;

use rust_nes::{apu::portaudio_player::PortAudioPlayer, controller, emulator, logger};

const DEFAULT_SCREEN_SCALE: &str = "2.0";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "assets/test.nes")]
  rom_path: String,

  #[clap(short, long, default_value = "assets/keybindings.ini")]
  key_binding_path: String,

  #[clap(short, long, default_value = DEFAULT_SCREEN_SCALE)]
  scale: f32,
}

fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let args = Args::parse();
  let mut emulator = emulator::Emulator::new(args.scale);
  let (p1_key, p2_key) = controller::key_binding_parser::parse_key_binding(&args.key_binding_path);
  let mut player = PortAudioPlayer::new();
  player.init().unwrap();

  emulator.set_keys(p1_key, p2_key);
  emulator.run(&args.rom_path);
}
