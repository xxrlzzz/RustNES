use std::sync::{Arc, Mutex, mpsc};

use rust_emu_common::{emulator::RuntimeConfig, instance::{FrameBuffer, Instance, RunningStatus}, types::*};

use crate::{bus::MainBus, cartridge::GBACartridge, cpu::GBCpu, mapper::create_mapper, ppu::GBAPpu};

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt(Byte),
  PpuRender(),
}

// pub fn init_rom_from_data(
//   rom_data: &[u8],
//   runtime_config: &RuntimeConfig,
// ) -> Option<Box<GBAInstance>> {
//   let mut cartridge = GBACartridge::new();
//   if !cartridge.load_from_data(rom_data) {
//     return None;
//   }

//   GBAInstance::init_rom(cartridge, runtime_config)
// }

pub fn init_rom_from_path(
  rom_path: &str,
  runtime_config: &RuntimeConfig,
) -> Option<Box<GBAInstance>> {
  let mut cartridge = GBACartridge::new();
  if !cartridge.load_from_file(rom_path) {
    return None;
  }
  GBAInstance::init_rom(cartridge)
}


pub struct GBAInstance {
  pub(crate) cpu: Arc<Mutex<GBCpu>>,
  pub(crate) ppu: Arc<Mutex<GBAPpu>>,
  pub(crate) stat: RunningStatus,
  pub(crate) message_rx: mpsc::Receiver<Message>,
  pub(crate) rgba: Option<FrameBuffer>,
}

impl GBAInstance {
  pub(crate) fn new(
    cpu: Arc<Mutex<GBCpu>>,
    ppu: Arc<Mutex<GBAPpu>>,
    message_rx: mpsc::Receiver<Message>,
  ) -> Self {
    Self {
      cpu,
      ppu,
      message_rx,
      stat: RunningStatus::Running,
      rgba: None,
    }
  }
}

impl Instance for GBAInstance {
    fn step(&mut self) -> u32 {
      let circle = {
        let mut cpu = self.cpu.lock().unwrap();
        let circle = cpu.step();
        circle
      };
      // log::info!("step {}", circle);
      for _ in 0..circle {
        for _ in 0..4 {
          self.ppu.lock().unwrap().step();
        }
      }
      self.consume_message();
      circle
    }

    fn consume_message(&mut self) {
      // TODO
    }

    fn can_run(&self) -> bool {
      self.stat.is_focusing() && !self.stat.is_pausing()
    }

    fn take_rgba(&mut self) -> Option<FrameBuffer> {
      self.rgba.take()
    }

    fn stop(&mut self) {
      // self.apu.lock().unwrap().stop();
    }

    fn do_save(&self, file: &String) {
      // match self.save(file) {
      //   Ok(_) => info!("save success"),
      //   Err(e) => error!("save failed: {}", e),
      // }
    }

    fn focus(&mut self) {
      self.stat.focus();
    }

    fn unfocus(&mut self) {
      self.stat.unfocus();
    }

    fn is_pausing(&mut self) -> bool {
      self.stat.is_pausing()
    }

    fn toggle_pause(&mut self) {
      self.stat.toggle_pause()
    }
}

impl GBAInstance {
  pub fn init_rom(cart: GBACartridge) -> Option<Box<Self>> {
    let (message_sx, message_rx) = mpsc::channel::<Message>();    
    let ppu = Arc::new(Mutex::new(GBAPpu::new(message_sx.clone())));

    let mut main_bus = MainBus::new();
    // main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());

    let mut cpu = GBCpu::new(main_bus);
    let mapper = create_mapper(cart);
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();

    let instance = Self::new(Arc::new(Mutex::new(cpu)), ppu, message_rx);

    Some(Box::new(instance))
  }
}