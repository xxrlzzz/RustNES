use std::{
  cell::RefCell,
  fs::OpenOptions,
  io::Write,
  rc::Rc,
  time::{Duration, Instant},
};

use log::{error, info};
use serde_json::json;

use crate::{
  apu::Apu,
  bus::{main_bus::MainBus, message_bus::MessageBus, picture_bus::PictureBus},
  cartridge::Cartridge,
  common::{sample_profile, MatrixType},
  cpu::Cpu,
  emulator::RuntimeConfig,
  mapper::factory,
  ppu::Ppu,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RunningStatus {
  Running = 0,
  Pause = 1,
  LostFocus = 2,
  PauseAndLostFocus = 3,
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
  pub(crate) apu: Rc<RefCell<Apu>>,
  pub(crate) cpu: Cpu,
  pub(crate) ppu: Rc<RefCell<Ppu>>,
  pub(crate) stat: RunningStatus,
  pub(crate) cycle_timer: Instant,
  pub(crate) elapsed_time: Duration,
}

impl Instance {
  pub(crate) fn new(apu: Rc<RefCell<Apu>>, cpu: Cpu, ppu: Rc<RefCell<Ppu>>) -> Self {
    Self {
      apu,
      cpu,
      ppu,
      stat: RunningStatus::Running,
      cycle_timer: Instant::now(),
      elapsed_time: Duration::new(0, 0),
    }
  }

  fn init_rom(
    cartridge: Cartridge,
    runtime_config: &RuntimeConfig,
    message_bus: Rc<RefCell<MessageBus>>,
  ) -> Option<Self> {
    let ppu = Rc::new(RefCell::new(Ppu::new(PictureBus::new(), message_bus)));
    let apu = Rc::new(RefCell::new(Apu::new()));
    let mut main_bus = MainBus::new(apu.clone(), ppu.clone());
    main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());

    let mut cpu = Cpu::new(main_bus);
    let mapper = factory::create_mapper(cartridge);
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();
    ppu.borrow_mut().set_mapper_for_bus(mapper);
    // ppu.borrow_mut().reset();

    apu.borrow_mut().start();
    Some(Self::new(apu, cpu, ppu))
  }

  pub(crate) fn init_rom_from_data(
    rom_data: &[u8],
    runtime_config: &RuntimeConfig,
    message_bus: Rc<RefCell<MessageBus>>,
  ) -> Option<Self> {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_data(rom_data) {
      return None;
    }

    Self::init_rom(cartridge, runtime_config, message_bus)
  }

  pub(crate) fn init_rom_from_path(
    rom_path: &str,
    runtime_config: &RuntimeConfig,
    message_bus: Rc<RefCell<MessageBus>>,
  ) -> Option<Self> {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_file(rom_path) {
      return None;
    }
    Self::init_rom(cartridge, runtime_config, message_bus)
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

  pub(crate) fn update_timer(&mut self) {
    let now = Instant::now();
    self.elapsed_time += now - self.cycle_timer;
    self.cycle_timer = now;
  }

  pub(crate) fn do_save(&self, file: &String) {
    match self.save(file) {
      Ok(_) => info!("save success"),
      Err(e) => error!("save failed: {}", e),
    }
  }

  fn save(&self, file: &String) -> Result<(), std::io::Error> {
    let path = std::path::Path::new(file);
    {
      let dir = path.parent().expect("invalid path");
      if !dir.exists() {
        std::fs::create_dir(dir)?;
      }
    }
    let mut file = std::fs::File::create(path)?;

    let json = json!({
      "apu": serde_json::to_string(&*self.apu).unwrap(),
      "cpu": serde_json::to_string(&self.cpu).unwrap(),
      "ppu": serde_json::to_string(&*self.ppu).unwrap(),
      "main_bus" : self.cpu.main_bus().save(),
    });
    file.write_all(json.to_string().as_bytes())
  }

  pub(crate) fn load(
    runtime_config: &RuntimeConfig,
    message_bus: Rc<RefCell<MessageBus>>,
  ) -> Result<Self, std::io::Error> {
    let file = OpenOptions::new()
      .read(true)
      .open(&runtime_config.save_path)?;

    let json_obj: serde_json::Value = serde_json::from_reader(file)?;
    fn str_mapper(json_value: &serde_json::Value) -> &str {
      json_value.as_str().unwrap()
    }
    let mut cpu: Cpu = json_obj
      .get("cpu")
      .map(str_mapper)
      .map(|cpu_str| serde_json::from_str(cpu_str).unwrap())
      .unwrap();
    let apu = json_obj
      .get("apu")
      .map(str_mapper)
      .map(|apu_str| {
        let mut apu: Apu = serde_json::from_str(apu_str).unwrap();
        apu.start();
        Rc::new(RefCell::new(apu))
      })
      .unwrap();
    let ppu = json_obj
      .get("ppu")
      .map(str_mapper)
      .map(|ppu_str| {
        let mut ppu: Ppu = serde_json::from_str(ppu_str).unwrap();
        ppu.set_message_bus(message_bus);
        Rc::new(RefCell::new(ppu))
      })
      .unwrap();
    json_obj.get("main_bus").map(|main_bus| {
      let mut main_bus = MainBus::load(main_bus, ppu.clone(), apu.clone());
      main_bus.set_controller_keys(runtime_config.ctl1.clone(), runtime_config.ctl2.clone());
      cpu.set_main_bus(main_bus);
    });
    Ok(Instance::new(apu, cpu, ppu))
  }

  pub(crate) fn step(&mut self, matrix: &mut MatrixType) {
    let mut now = Instant::now();
    {
      let mut ppu = self.ppu.borrow_mut();
      ppu.step();
      ppu.step();
      ppu.step();
    }
    sample_profile(&mut now, "ppu", matrix);

    self.cpu.step();
    sample_profile(&mut now, "cpu", matrix);

    self.apu.borrow_mut().step();
    sample_profile(&mut now, "apu", matrix);
  }
}
