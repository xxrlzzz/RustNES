use std::sync::{Arc, Mutex, mpsc};

use image::RgbaImage;
use rust_emu_common::{emulator::RuntimeConfig, instance::{FrameBuffer, Instance, RunningStatus}, types::*};

use crate::{bus::GBAMainBus, cartridge::GBACartridge, cpu::GBCpu, mapper::create_mapper, ppu::GBAPpu, timer::Timer};

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt(Byte),
  PpuRender(RgbaImage),
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
  GBAInstance::init_rom(cartridge, runtime_config)
}


pub struct GBAInstance {
  pub(crate) cpu: Arc<Mutex<GBCpu>>,
  pub(crate) ppu: Arc<Mutex<GBAPpu>>,
  pub(crate) timer: Arc<Mutex<Timer>>,
  pub(crate) stat: RunningStatus,
  pub(crate) message_rx: mpsc::Receiver<Message>,
  pub(crate) rgba: Option<FrameBuffer>,
}

impl GBAInstance {
  pub(crate) fn new(
    cpu: Arc<Mutex<GBCpu>>,
    ppu: Arc<Mutex<GBAPpu>>,
    timer: Arc<Mutex<Timer>>,
    message_rx: mpsc::Receiver<Message>,
  ) -> Self {
    Self {
      cpu,
      ppu,
      timer,
      message_rx,
      stat: RunningStatus::Running,
      rgba: None,
    }
  }
}

impl<'a> Instance for GBAInstance {
    fn step(&mut self) -> u32 {
      let circle = {
        let mut cpu = self.cpu.lock().unwrap();
        let circle = cpu.step();
        circle
      };
      // log::info!("step {}", circle);
      {
        let mut ppu = self.ppu.lock().unwrap();
        let mut timer = self.timer.lock().unwrap();
        for _ in 0..circle {
          for _ in 0..4 {
            ppu.step();
            timer.step();
          }
          // dma_tick();
        }
      }
      self.consume_message();
      circle as u32
    }

    fn consume_message(&mut self) {
      while let Ok(message) = self.message_rx.try_recv() {
        match message {
          Message::CpuInterrupt(interrupt) => {
            self.cpu.lock().unwrap().trigger_interrupt(interrupt);
          },
          Message::PpuRender(frame) => {
            log::info!("handle frame ");
            self.rgba = Some(frame);
          }
        }
      }
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
  pub fn init_rom(cart: GBACartridge, runtime_config: &RuntimeConfig) -> Option<Box<Self>> {
    let (message_sx, message_rx) = mpsc::channel::<Message>();    
    let ppu = Arc::new(Mutex::new(GBAPpu::new(message_sx.clone())));
    let timer = Arc::new(Mutex::new(Timer::new(message_sx.clone())));
    let mut main_bus = GBAMainBus::new(ppu.clone(), timer.clone());
    main_bus.set_controller_keys(runtime_config.ctl1.clone());

    let mut cpu = GBCpu::new(main_bus);
    let mapper = create_mapper(cart);
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();

    ppu.lock().unwrap().set_mapper_for_bus(mapper);

    let instance = Self::new(Arc::new(Mutex::new(cpu)), ppu, timer, message_rx);

    Some(Box::new(instance))
  }
}