use core::time;

#[cfg(feature = "use_sdl2")]
mod sdl2;

#[cfg(feature = "use_gl")]
mod gl;

#[cfg(target_arch = "wasm32")]
mod web;

use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, info};

use crate::instance::Instance;
use crate::instant::Instant;

pub const CPU_FREQUENCY: u32 = 1789773;
use crate::controller::key_binding_parser::KeyType;
const NES_VIDEO_WIDTH: u32 = (256) as u32;
const NES_VIDEO_HEIGHT: u32 = (240) as u32;
const CPU_CYCLE_DURATION: Duration = Duration::from_nanos(559);

pub const APP_NAME: &str = "NES-Simulator";

const FRAME_DURATION: Duration = time::Duration::from_millis(16);

#[derive(Clone)]
pub struct RuntimeConfig {
  pub save_path: String,
  pub screen_scale: f32,

  pub ctl1: Vec<KeyType>,
  pub ctl2: Vec<KeyType>,
}

pub struct Emulator {
  pub runtime_config: RuntimeConfig,
  pub(crate) cycle_timer: Instant,
  pub(crate) elapsed_time: Duration,
}

impl RuntimeConfig {
  pub fn window_size(&self) -> (u32, u32) {
    let width = (NES_VIDEO_WIDTH as f32 * self.screen_scale) as u32;
    let height = (NES_VIDEO_HEIGHT as f32 * self.screen_scale) as u32;
    (width, height)
  }
}


impl Emulator {
  pub fn new(screen_scale: f32, save_path: String, ctl1: Vec<KeyType>, ctl2: Vec<KeyType>) -> Self {
    Self {
      runtime_config: RuntimeConfig {
        save_path,
        screen_scale,
        ctl1,
        ctl2,
      },
      cycle_timer: Instant::now(),
      elapsed_time: Duration::new(0, 0),
    }
  }

  pub(crate) fn update_timer(&mut self) {
    let now = Instant::now();
    self.elapsed_time += now - self.cycle_timer;
    self.cycle_timer = now;
  }

  fn one_frame(&mut self, instance: &mut Box<dyn Instance>) -> Duration {
    let mut iter_time = CPU_FREQUENCY;

    self.update_timer();
    // 1789773 / 1000 * 16
    let mut iters = 0;
    for i in 0.. {
      let cur_circle = instance.step();
      iters += cur_circle;
      if i % 100 == 0 && Instant::now() - self.cycle_timer > FRAME_DURATION {
        iter_time = iters;
        break;
      }
      let duration = CPU_CYCLE_DURATION * cur_circle;
      if iters > CPU_FREQUENCY / 60 || self.elapsed_time < duration {
        iter_time = iters;
        break;
      }
    }
    let cost = Instant::now() - self.cycle_timer;
    debug!("last frame toke {:?} for {} times.", cost, iter_time);
    cost
  }

  #[cfg(not(any(feature = "use_sdl2", feature = "use_gl")))]
  pub fn run(&mut self, mut _instance: Instance) {
    info!("start running");
  }
}

#[cfg(test)]
mod tests {
  use super::Emulator;
  // use crate::{controller, instance::NESInstance};
  // use rust_emu_common::instance::Instance;
  // use rust_emu_common::logger;
  // use std::{
  //   fs::{self},
  //   path::Path,
  // };

  // fn create_dummy_emulator() -> (Emulator, Box<NESInstance>) {
  //   let (p1_key, p2_key) =
  //     controller::key_binding_parser::parse_key_binding("assets/keybindings.ini");
  //   let emulator = Emulator::new(2.0, "tmp".to_string(), p1_key, p2_key);
  //   let instance = emulator.create_instance("assets/mario.nes");
  //   (emulator, instance)
  // }

  // #[test]
  // fn save_load_test() {
  //   match logger::init() {
  //     Err(_) => return,
  //     Ok(_) => {}
  //   };
  //   let (emulator, instance) = create_dummy_emulator();
  //   instance.do_save(&"tmp".to_string());

  //   NESInstance::load(&emulator.runtime_config).unwrap();
  //   fs::remove_file(Path::new("tmp")).unwrap();
  // }
}
