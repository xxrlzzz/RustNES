// #![cfg(target_os = "android")]

use std::{ffi::CStr, os::raw::c_char};

use clap::Parser;
use log::info;

use crate::{controller, emulator};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "assets/mario.nes")]
  rom_path: String,

  #[clap(short, long, default_value = "assets/keybindings.ini")]
  key_binding_path: String,

  #[clap(short, long, default_value = "2.0")]
  scale: f32,

  #[clap(long, default_value = "save/saved.json")]
  save_path: String,
}

#[no_mangle]
pub extern "C" fn android_main(argc: std::os::raw::c_int, argv: *const *const u8) {
  android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Trace));
  let mario = include_bytes!("../assets/mario.nes");

  info!("enter android_main");
  let (p1_key, p2_key) = controller::key_binding_parser::default_key_binding();
  let mut emulator = emulator::Emulator::new(1.0, ".".into(), p1_key, p2_key);

  let _args: Vec<&[u8]> = unsafe {
    (0..argc)
      .map(|i| {
        let cstr = CStr::from_ptr(*argv.offset(i as _) as *const c_char);
        // OsStringExt::from_vec(cstr.to_bytes().to_vec())
        // cstr.to_string_lossy().into_owned()
        cstr.to_bytes()
      })
      .collect()
  };
  // info!("{:?}", args);
  // info!("rom_path: {}\n rom_data: {}", rom_path, rom_data);
  let instance = emulator.create_instance_from_data(mario);
  emulator.run(instance);
  info!("emulator exit");
}
