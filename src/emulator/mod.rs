use core::time;

#[cfg(feature = "use_sdl2")]
mod sdl2;

#[cfg(feature = "use_gl")]
mod gl;

use std::time::{Duration, Instant};

use log::info;

use crate::apu::CPU_FREQUENCY;
use crate::controller::key_binding_parser::KeyType;
use crate::instance::Instance;
use crate::ppu::{SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES};

const NES_VIDEO_WIDTH: u32 = SCANLINE_VISIBLE_DOTS as u32;
const NES_VIDEO_HEIGHT: u32 = VISIBLE_SCANLINES as u32;
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

impl RuntimeConfig {
  pub fn window_size(&self) -> (u32, u32) {
    let width = (NES_VIDEO_WIDTH as f32 * self.screen_scale) as u32;
    let height = (NES_VIDEO_HEIGHT as f32 * self.screen_scale) as u32;
    (width, height)
  }
}

pub struct Emulator {
  runtime_config: RuntimeConfig,
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
    }
  }

  fn one_frame(&mut self, instance: &mut Instance) -> Duration {
    let mut iter_time = CPU_FREQUENCY;
    instance.update_timer();
    // 1789773 / 1000 * 16
    for i in 0..28636 {
      instance.step();

      if i % 3000 == 0 && Instant::now() - instance.cycle_timer > FRAME_DURATION {
        iter_time = i;
        break;
      }
      if instance.elapsed_time < CPU_CYCLE_DURATION {
        iter_time = i;
        break;
      }
      instance.elapsed_time -= CPU_CYCLE_DURATION;
    }
    let cost = Instant::now() - instance.cycle_timer;
    info!("last frame toke {:?} for {} times.", cost, iter_time);

    cost
  }

  pub fn create_instance(&self, rom_path: &str) -> Instance {
    Instance::init_rom_from_path(rom_path, &self.runtime_config).expect("Failed to load rom.")
  }

  pub fn create_instance_from_data(&self, rom_data: &[u8]) -> Instance {
    Instance::init_rom_from_data(rom_data, &self.runtime_config).expect("Failed to load rom.")
  }

  #[cfg(not(any(feature = "use_sdl2", feature = "use_gl")))]
  pub fn run(&mut self, mut _instance: Instance) {
    info!("start running");
  }
}
