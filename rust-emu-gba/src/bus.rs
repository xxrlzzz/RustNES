use std::{
  cell::RefCell,
  rc::Rc,
  sync::{Arc, Mutex},
};

use rust_emu_common::{
  component::main_bus::{MainBus, RegisterHandler},
  controller::key_binding_parser::KeyType,
  mapper::Mapper,
  types::*,
};

use crate::controller::GBAController;

#[derive(Default)]
pub struct GBAMainBus {
  mapper: Option<Rc<RefCell<dyn Mapper>>>,

  wram: Vec<Byte>,
  hram: Vec<Byte>,

  io_serial_data: [Byte; 0x2],

  registers: Vec<Arc<Mutex<dyn RegisterHandler>>>,

  control: GBAController,
}

impl GBAMainBus {
  pub fn new(ppu: Arc<Mutex<dyn RegisterHandler>>, timer: Arc<Mutex<dyn RegisterHandler>>) -> Self {
    Self {
      wram: vec![0; 0x2000],
      hram: vec![0; 0x80],
      mapper: None,

      io_serial_data: [0; 0x2],

      registers: vec![ppu, timer],

      control: GBAController::new(),
    }
  }

  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    self.mapper = Some(mapper);
  }

  fn io_read(&mut self, addr: Address) -> Byte {
    // TODO
    match addr {
      0xFF00 => {
        // game pad
        self.control.read()
        // 0xFF
      }
      0xFF01 => self.io_serial_data[0],
      0xFF02 => self.io_serial_data[1],
      0xFF04..=0xFF07 => {
        // timer
        self.registers[1].lock().unwrap().read(addr).unwrap()
      }
      0xFF0F => {
        // interrupt
        // log::warn!("interrupt read");
        0xFF
      }
      0xFF10..=0xFF3F => {
        // sound
        0xFF
      }
      0xFF40..=0xFF4B => {
        // ppu
        self.registers[0].lock().unwrap().read(addr).unwrap()
      }
      _ => {
        // no impl
        0
      }
    }
  }

  fn io_write(&mut self, addr: Address, value: Byte) {
    match addr {
      0xFF00 => {
        // game pad
        self.control.strobe(value)
      }
      0xFF01 => {
        self.io_serial_data[0] = value;
      }
      0xFF02 => {
        self.io_serial_data[1] = value;
      }
      0xFF04..=0xFF07 => {
        // timer
        self.registers[1].lock().unwrap().write(addr, value);
      }
      0xFF0F => {
        // interrupt
        // log::warn!("interrupt write {}", value);
      }
      0xFF10..=0xFF3F => {
        // sound
      }
      0xFF40..=0xFF4B => {
        self.registers[0].lock().unwrap().write(addr, value);
      }
      _ => {
        // no impl
      }
    }
  }

  fn wram_read(&self, addr: Address) -> Byte {
    if addr >= 0xE000 || addr < 0xC000 {
      log::error!("WRAM read out of range {}", addr);
      return 0;
    }
    self.wram[addr as usize - 0xC000]
  }

  fn wram_write(&mut self, addr: Address, value: Byte) {
    if addr >= 0xE000 || addr < 0xC000 {
      log::error!("WRAM write out of range {}", addr);
      return;
    }
    self.wram[addr as usize - 0xC000] = value;
  }

  fn hram_read(&self, addr: Address) -> Byte {
    self.hram[addr as usize - 0xFF80]
  }
  fn hram_write(&mut self, addr: Address, value: Byte) {
    self.hram[addr as usize - 0xFF80] = value;
  }

  pub fn set_controller_keys(&mut self, p1: Vec<KeyType>) {
    self.control.set_key_bindings(p1);
  }
}

impl MainBus for GBAMainBus {
  /**
   *
   * 0x0000 - 0x3FFF : ROM Bank 0
   * 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
   * 0x8000 - 0x97FF : CHR RAM
   * 0x9800 - 0x9BFF : BG Map 1
   * 0x9C00 - 0x9FFF : BG Map 2
   * 0xA000 - 0xBFFF : Cartridge RAM
   * 0xC000 - 0xCFFF : RAM Bank 0
   * 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
   * 0xE000 - 0xFDFF : Reserved - Echo RAM
   * 0xFE00 - 0xFE9F : Object Attribute Memory
   * 0xFEA0 - 0xFEFF : Reserved - Unusable
   * 0xFF00 - 0xFF7F : I/O Registers
   * 0xFF80 - 0xFFFE : Zero Page
   */
  fn read(&mut self, addr: Address) -> Byte {
    return match addr {
      // ROM data
      0x0000..=0x7FFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
      // Char/Map data
      0x8000..=0x9FFF => self.mapper.as_ref().unwrap().borrow().read_chr(addr),
      // Cartridge RAM
      0xA000..=0xBFFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
      // WRAM (Working RAM)
      0xC000..=0xDFFF => self.wram_read(addr),
      // reserved echo ram...
      0xE000..=0xFDFF => 0,
      0xFE00..=0xFE9F => {
        //OAM handle by PPU
        self.registers[0].lock().unwrap().read(addr).unwrap()
      }
      0xFEA0..=0xFEFF => {
        //reserved unusable...
        0
      }
      //IO Registers...
      0xFF00..=0xFF7F => self.io_read(addr),
      //CPU enable register...
      // handle by cpu itself
      0xFFFF => 0,
      // no impl
      _ => self.hram_read(addr),
    };
  }

  fn write(&mut self, addr: Address, data: Byte) {
    match addr {
      // ROM data
      0x0000..=0x7FFF => self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .write_prg(addr, data),
      // Char/Map data
      0x8000..=0x9FFF => self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .write_chr(addr, data),
      // Cartridge RAM
      0xA000..=0xBFFF => self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .write_prg(addr, data),
      // WRAM (Working RAM)
      0xC000..=0xDFFF => self.wram_write(addr, data),
      // reserved echo ram...
      0xE000..=0xFDFF => (),
      0xFE00..=0xFE9F => {
        //OAM handle by PPU
        self.registers[0].lock().unwrap().write(addr, data);
      }
      0xFEA0..=0xFEFF => {
        //reserved unusable...
        ()
      }
      //IO Registers...
      0xFF00..=0xFF7F => self.io_write(addr, data),
      //CPU enable register...
      // handle by cpu itself
      0xFFFF => (),
      // no impl
      _ => self.hram_write(addr, data),
    }
  }
}
