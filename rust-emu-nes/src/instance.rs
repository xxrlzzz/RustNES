use std::{
  fs::OpenOptions,
  io::{BufReader, BufWriter, Write},
  sync::{Arc, Mutex},
};

use ciborium::{de::from_reader, ser::into_writer};
use image::RgbaImage;
use log::{error, info, warn};
use rust_emu_common::{emulator::RuntimeConfig, instance::{FrameBuffer, Instance, RunningStatus}};
use std::sync::mpsc;

use crate::{
  apu::Apu,
  bus::{main_bus::MainBus, message_bus::Message},
  cartridge::NESCartridge,
  cpu::{Cpu, InterruptType},
  mapper::factory,
  ppu::{Ppu, SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES},
};

pub fn init_rom_from_data(
  rom_data: &[u8],
  runtime_config: &RuntimeConfig,
) -> Option<Box<NESInstance>> {
  let mut cartridge = NESCartridge::new();
  if !cartridge.load_from_data(rom_data) {
    return None;
  }

  NESInstance::init_rom(cartridge, runtime_config)
}

pub fn init_rom_from_path(
  rom_path: &str,
  runtime_config: &RuntimeConfig,
) -> Option<Box<NESInstance>> {
  let mut cartridge = NESCartridge::new();
  if !cartridge.load_from_file(rom_path) {
    return None;
  }
  NESInstance::init_rom(cartridge, runtime_config)
}

pub struct NESInstance {
  pub(crate) apu: Arc<Mutex<Apu>>,
  pub(crate) cpu: Arc<Mutex<Cpu>>,
  pub(crate) ppu: Arc<Mutex<Ppu>>,
  pub(crate) stat: RunningStatus,
  pub(crate) message_rx: mpsc::Receiver<Message>,
  pub(crate) rgba: Option<FrameBuffer>,
}

impl NESInstance {
  pub(crate) fn new(
    apu: Arc<Mutex<Apu>>,
    cpu: Arc<Mutex<Cpu>>,
    ppu: Arc<Mutex<Ppu>>,
    message_rx: mpsc::Receiver<Message>,
  ) -> Self {
    Self {
      apu,
      cpu,
      ppu,
      message_rx,
      stat: RunningStatus::Running,
      rgba: None,
    }
  }
}
impl Instance for NESInstance {
  fn consume_message(&mut self) {
    while let Ok(message) = self.message_rx.try_recv() {
      match message {
        Message::CpuInterrupt(interrupt) => {
          self.cpu.lock().unwrap().trigger_interrupt(interrupt);
        }
        Message::PpuRender(frame) => {
          self.rgba = Some(frame);
        }
      };
    }
  }

  fn toggle_pause(&mut self) {
    self.stat.toggle_pause()
  }

  fn can_run(&self) -> bool {
    self.stat.is_focusing() && !self.stat.is_pausing()
  }

  fn take_rgba(&mut self) -> Option<FrameBuffer> {
    self.rgba.take()
  }

  fn step(&mut self) -> u32 {
    let circle = {
      let mut cpu = self.cpu.lock().unwrap();
      let circle = cpu.step();
      cpu.reset_skip_cycles();
      circle
    };
    {
      let mut ppu = self.ppu.lock().unwrap();
      for _ in 0..circle {
        ppu.step();
        ppu.step();
        ppu.step();
      }
    }
    {
      let mut apu = self.apu.lock().unwrap();
      for _ in 0..circle {
        apu.step();
      }
    }
    self.consume_message();

    circle
  }

  fn stop(&mut self) {
    self.apu.lock().unwrap().stop();
  }

  /// Save and load
  fn do_save(&self, file: &String) {
    match self.save(file) {
      Ok(_) => info!("save success"),
      Err(e) => error!("save failed: {}", e),
    }
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
}

impl NESInstance {
  pub fn save(&self, file: &String) -> Result<(), std::io::Error> {
    let path = std::path::Path::new(file);
    log::info!("save to {}", path.display());
    {
      let dir = path.parent().expect("invalid path");
      if !dir.exists() && dir != std::path::Path::new("") {
        log::debug!("create dir {:?}", dir);
        std::fs::create_dir(dir)?;
      }
    }
    let mut file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(&mut file);
    into_writer(&*self.cpu.lock().unwrap(), &mut writer).unwrap();
    into_writer(&*self.apu.lock().unwrap(), &mut writer).unwrap();
    into_writer(&*self.ppu.lock().unwrap(), &mut writer).unwrap();
    self
      .cpu
      .as_ref()
      .lock()
      .unwrap()
      .main_bus()
      .save_binary(writer)
      .flush()?;
    // writer.flush()
    Ok(())
  }

  fn init_rom(cartridge: NESCartridge, runtime_config: &RuntimeConfig) -> Option<Box<Self>> {
    let (message_sx, message_rx) = mpsc::channel::<Message>();
    let ppu = Arc::new(Mutex::new(Ppu::new(message_sx.clone())));

    let apu = Arc::new(Mutex::new(Apu::new(message_sx.clone())));
    let mut main_bus = MainBus::new(apu.clone(), ppu.clone());
    main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());

    let mut cpu = Cpu::new(main_bus);
    let ppu_clone = ppu.clone();
    let mapper = factory::create_mapper(
      cartridge,
      Box::new(move |val: u8| {
        let r = ppu_clone.try_lock();
        if r.is_ok() {
          r.unwrap().update_mirroring(Some(val));
        } else {
          warn!("ppu is locked");
        }
      }),
      Box::new(move || {
        if let Err(e) = message_sx.send(Message::CpuInterrupt(InterruptType::IRQ)) {
          log::error!("send interrupt error {:?}", e);
        }
      }),
    );
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();
    let cpu = Arc::new(Mutex::new(cpu));
    ppu.lock().unwrap().set_mapper_for_bus(mapper);

    {
      let cpu_clone = cpu.clone();
      let mut apu = apu.lock().unwrap();
      apu.set_read_cb(Box::new(move |addr| {
        let mut inner_cpu = cpu_clone.lock().unwrap();
        inner_cpu.skip_dmc_cycles();
        inner_cpu.main_bus_mut().read(addr)
      }));
      apu.start();
    }

    // ppu.borrow_mut().reset();
    let instance = Self::new(apu, cpu, ppu, message_rx);

    Some(Box::new(instance))
  }

  pub fn load(runtime_config: &RuntimeConfig) -> Result<Self, std::io::Error> {
    let file = OpenOptions::new()
      .read(true)
      .open(&runtime_config.save_path)?;
    let mut reader = BufReader::new(file);
    let (message_sx, message_rx) = mpsc::channel::<Message>();

    let mut cpu: Cpu = from_reader(&mut reader).unwrap();
    let apu = from_reader(&mut reader)
      .map(|mut apu: Apu| {
        apu.set_message_bus(message_sx.clone());
        apu.start();

        Arc::new(Mutex::new(apu))
      })
      .unwrap();
    let ppu = from_reader(&mut reader)
      .map(|mut ppu: Ppu| {
        ppu.set_message_bus(message_sx.clone());
        ppu.image = RgbaImage::new(SCANLINE_VISIBLE_DOTS as u32, VISIBLE_SCANLINES as u32);

        Arc::new(Mutex::new(ppu))
      })
      .unwrap();
    let mut main_bus = MainBus::load_binary(reader, message_sx, ppu.clone(), apu.clone());
    main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());
    cpu.set_main_bus(main_bus);
    let cpu = Arc::new(Mutex::new(cpu));
    let cpu_clone = cpu.clone();
    apu.lock().unwrap().set_read_cb(Box::new(move |addr| {
      let mut inner_cpu = cpu_clone.lock().unwrap();
      inner_cpu.skip_dmc_cycles();
      inner_cpu.main_bus_mut().read(addr)
    }));

    Ok(Self::new(apu, cpu, ppu, message_rx))
  }
}
