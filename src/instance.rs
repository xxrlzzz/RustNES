use std::{
  fs::OpenOptions,
  io::{BufReader, BufWriter, Write},
  sync::{Arc, Condvar, Mutex},
  time::Duration,
};

use ciborium::{de::from_reader, ser::into_writer};
use image::{ImageBuffer, Rgba, RgbaImage};
use log::{error, info, warn};
use std::sync::mpsc;

use crate::{
  apu::Apu,
  bus::{main_bus::MainBus, message_bus::Message},
  cartridge::Cartridge,
  common::instant::Instant,
  cpu::Cpu,
  emulator::RuntimeConfig,
  mapper::factory,
  ppu::{Ppu, SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES},
};

pub type FrameBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RunningStatus {
  Running = 0,
  Pause = 1,
  LostFocus = 2,
  PauseAndLostFocus = 3,
  Exist = 4,
  PPURun = 5,
}

impl From<u8> for RunningStatus {
  fn from(v: u8) -> Self {
    match v {
      0 => Self::Running,
      1 => Self::Pause,
      2 => Self::LostFocus,
      3 => Self::PauseAndLostFocus,
      _ => panic!(""),
    }
  }
}
impl RunningStatus {
  pub(crate) fn unpause(&mut self) {
    *self = Self::from(*self as u8 & Self::LostFocus as u8)
  }

  pub(crate) fn pause(&mut self) {
    *self = Self::from(*self as u8 | Self::Pause as u8)
  }

  pub(crate) fn is_pausing(&self) -> bool {
    (*self as u8 & Self::Pause as u8) != 0
  }

  pub(crate) fn unfocus(&mut self) {
    *self = Self::from(*self as u8 | Self::LostFocus as u8);
  }

  pub(crate) fn focus(&mut self) {
    *self = Self::from(*self as u8 & Self::Pause as u8);
  }

  pub(crate) fn is_focusing(&self) -> bool {
    (*self as u8 & Self::LostFocus as u8) == 0
  }
}
pub struct Instance {
  pub(crate) apu: Arc<Mutex<Apu>>,
  pub(crate) cpu: Cpu,
  pub(crate) ppu: Arc<Mutex<Ppu>>,
  pub(crate) stat: RunningStatus,
  pub(crate) cycle_timer: Instant,
  pub(crate) elapsed_time: Duration,
  pub(crate) message_rx: mpsc::Receiver<Message>,
  pub(crate) ppu_cond: Arc<(Mutex<RunningStatus>, Condvar)>,
  pub(crate) rgba: Option<FrameBuffer>,
}

impl Instance {
  pub(crate) fn new(
    apu: Arc<Mutex<Apu>>,
    cpu: Cpu,
    ppu: Arc<Mutex<Ppu>>,
    message_rx: mpsc::Receiver<Message>,
    ppu_cond: Arc<(Mutex<RunningStatus>, Condvar)>,
  ) -> Self {
    Self {
      apu,
      cpu,
      ppu,
      message_rx,
      stat: RunningStatus::Running,
      ppu_cond,
      cycle_timer: Instant::now(),
      elapsed_time: Duration::new(0, 0),
      rgba: None,
    }
  }

  pub(crate) fn consume_message(&mut self) {
    while let Ok(message) = self.message_rx.try_recv() {
      match message {
        Message::CpuInterrupt(interrupt) => {
          self.cpu.interrupt(interrupt);
        }
        Message::PpuRender(frame) => {
          self.rgba = Some(frame);
        }
      };
    }
  }

  pub(crate) fn toggle_pause(&mut self) {
    if self.stat.is_focusing() {
      self.cycle_timer = Instant::now();
      self.stat.unpause();
    } else {
      self.stat.pause();
    }
  }

  pub(crate) fn can_run(&self) -> bool {
    self.stat.is_focusing() && !self.stat.is_pausing()
  }

  pub(crate) fn take_rgba(&mut self) -> Option<FrameBuffer> {
    self.rgba.take()
  }

  pub(crate) fn update_timer(&mut self) {
    let now = Instant::now();
    self.elapsed_time += now - self.cycle_timer;
    self.cycle_timer = now;
  }

  // If running on multi-thread, render ppu on different thread.
  // Disabled due to sync with performance issue.
  #[allow(dead_code)]
  pub(crate) fn launch_ppu(
    ppu: Arc<Mutex<Ppu>>,
    cond: Arc<(Mutex<RunningStatus>, Condvar)>,
  ) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
      let (lock, cvar) = &*cond;
      {
        let mut started = lock.lock().unwrap();
        while *started == RunningStatus::Pause {
          started = cvar.wait(started).unwrap();
        }
        if *started == RunningStatus::Exist {
          break;
        }
        *started = RunningStatus::PPURun;
      }
      let ppu = ppu.clone();
      {
        let mut ppu = ppu.lock().unwrap();

        ppu.step();
        ppu.step();
        ppu.step();
      }
      {
        let mut started = lock.lock().unwrap();
        *started = RunningStatus::Running;
        cvar.notify_one();
      }
    })
  }

  pub(crate) fn step(&mut self) {
    {
      let mut ppu = self.ppu.lock().unwrap();
      ppu.step();
      ppu.step();
      ppu.step();
    }
    self.consume_message();
    self.cpu.step();
    self.apu.lock().unwrap().step();
  }

  pub fn stop(&mut self) {
    let (lock, cvar) = &*self.ppu_cond;
    let mut started = lock.lock().unwrap();
    *started = RunningStatus::Exist;
    cvar.notify_one();

    self.apu.lock().unwrap().stop();
  }
}

/// Save and load
impl Instance {
  pub(crate) fn do_save(&self, file: &String) {
    match self.save(file) {
      Ok(_) => info!("save success"),
      Err(e) => error!("save failed: {}", e),
    }
  }

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
    into_writer(&self.cpu, &mut writer).unwrap();
    into_writer(&*self.apu.lock().unwrap(), &mut writer).unwrap();
    into_writer(&*self.ppu.lock().unwrap(), &mut writer).unwrap();
    writer = self.cpu.main_bus().save_binary(writer);
    writer.flush()
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
        ppu.set_message_bus(message_sx);
        ppu.image = RgbaImage::new(SCANLINE_VISIBLE_DOTS as u32, VISIBLE_SCANLINES as u32);

        Arc::new(Mutex::new(ppu))
      })
      .unwrap();
    let mut main_bus = MainBus::load_binary(reader, ppu.clone(), apu.clone());
    main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());
    cpu.set_main_bus(main_bus);

    let pair = Arc::new((Mutex::new(RunningStatus::Pause), Condvar::new()));
    #[cfg(not(feature = "wasm"))]
    Self::launch_ppu(ppu.clone(), pair.clone());

    Ok(Self::new(apu, cpu, ppu, message_rx, pair))
  }

  fn init_rom(cartridge: Cartridge, runtime_config: &RuntimeConfig) -> Option<Self> {
    let (message_sx, message_rx) = mpsc::channel::<Message>();
    let ppu = Arc::new(Mutex::new(Ppu::new(message_sx.clone())));
    let apu = Arc::new(Mutex::new(Apu::new(message_sx)));
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
    );
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();
    ppu.lock().unwrap().set_mapper_for_bus(mapper);
    // ppu.borrow_mut().reset();

    apu.lock().unwrap().start();

    let pair = Arc::new((Mutex::new(RunningStatus::Pause), Condvar::new()));
    #[cfg(not(feature = "wasm"))]
    Self::launch_ppu(ppu.clone(), pair.clone());
    Some(Self::new(apu, cpu, ppu, message_rx, pair))
  }

  pub(crate) fn init_rom_from_data(
    rom_data: &[u8],
    runtime_config: &RuntimeConfig,
  ) -> Option<Self> {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_data(rom_data) {
      return None;
    }

    Self::init_rom(cartridge, runtime_config)
  }

  pub(crate) fn init_rom_from_path(rom_path: &str, runtime_config: &RuntimeConfig) -> Option<Self> {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_file(rom_path) {
      return None;
    }
    Self::init_rom(cartridge, runtime_config)
  }
}
