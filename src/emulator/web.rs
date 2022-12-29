use log::{error, info};
use wasm_rgame::key_codes;

use super::{Emulator, RuntimeConfig};
use crate::controller::web_key::{self, WebEvent};
use crate::instance::Instance;

impl Emulator {
  pub fn frame(&mut self, instance: &mut Instance) -> Option<crate::instance::FrameBuffer> {
    let config = self.runtime_config.clone();
    for key in web_key::pull_events() {
      self.handle_event(&config, key, instance);
    }
    if instance.can_run() {
      self.one_frame(instance);
      instance.take_rgba()
    } else {
      None
    }
  }

  pub fn handle_event(
    &mut self,
    runtime_config: &RuntimeConfig,
    key: WebEvent,
    instance: &mut Instance,
  ) -> bool {
    match key {
      WebEvent::KeyDown(key_codes::Z) => {
        instance.do_save(&runtime_config.save_path);
      }
      WebEvent::KeyDown(key_codes::X) => match Instance::load(&runtime_config) {
        Ok(instance_load) => {
          *instance = instance_load;
          info!("load success")
        }
        Err(e) => error!("load failed: {}", e),
      },
      WebEvent::Focus(true) => instance.stat.focus(),
      WebEvent::Focus(false) => instance.stat.unfocus(),
      WebEvent::KeyDown(key_codes::F2) => instance.toggle_pause(),
      WebEvent::KeyDown(key_codes::F3) => {
        if instance.stat.is_pausing() {
          for _ in 0..29781 {
            instance.step();
          }
        }
      }
      WebEvent::KeyDown(key_codes::F4) => {
        log::set_max_level(log::LevelFilter::Debug);
        log::debug!("log switch into debug mode");
      }
      WebEvent::KeyDown(key_codes::F5) => {
        log::set_max_level(log::LevelFilter::Info);
        log::debug!("log switch into info mode");
      }
      WebEvent::KeyDown(key_codes::F6) => {
        log::set_max_level(log::LevelFilter::Warn);
        log::debug!("log switch into warn mode");
      }
      WebEvent::KeyDown(key_codes::F7) => {
        log::set_max_level(log::LevelFilter::Error);
        log::debug!("log switch into error mode");
      }
      _ => {}
    }
    true
  }
}
