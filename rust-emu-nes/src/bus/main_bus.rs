use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use log::{error, warn};
use rust_emu_common::controller::key_binding_parser::KeyType;
use rust_emu_common::controller::Controller;
use rust_emu_common::mapper::Mapper;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};
use rust_emu_common::types::*;

use crate::apu::Apu;
use crate::cpu::InterruptType;
use crate::mapper::factory::load_mapper;
use crate::ppu::Ppu;

use super::message_bus::Message;

pub type IORegister = u16;

pub const PPU_CTRL: IORegister = 0x2000;
pub const PPU_MASK: IORegister = 0x2001;
pub const PPU_STATUS: IORegister = 0x2002;
pub const OAM_ADDR: IORegister = 0x2003;
pub const OAM_DATA: IORegister = 0x2004;
pub const PPU_SCROL: IORegister = 0x2005;
pub const PPU_ADDR: IORegister = 0x2006;
pub const PPU_DATA: IORegister = 0x2007;
pub const OAM_DMA: IORegister = 0x4014;
pub const APU_ADDR: IORegister = 0x4015;
pub const JOY1: IORegister = 0x4016;
pub const JOY2: IORegister = 0x4017;

pub trait RegisterHandler {
  fn read(&mut self, address: IORegister) -> Option<Byte>;
  fn write(&mut self, address: IORegister, value: Byte) -> bool;
  fn dma(&mut self, page: *const Byte) -> bool;
}

#[derive(Default)]
pub struct MainBus {
  ram: Vec<Byte>,
  ext_ram: Vec<Byte>,
  has_ext_ram: bool,
  mapper: Option<Rc<RefCell<dyn Mapper>>>,
  registers: Vec<Arc<Mutex<dyn RegisterHandler>>>,
  control1: Controller,
  control2: Controller,

  skip_dma_cycles: bool,
}

impl MainBus {
  pub fn new<'a>(apu: Arc<Mutex<Apu>>, ppu: Arc<Mutex<Ppu>>) -> Self {
    Self {
      ram: vec![0; 0x800],
      ext_ram: vec![],
      has_ext_ram: false,
      mapper: None,
      registers: vec![ppu, apu],
      control1: Controller::new(),
      control2: Controller::remote_controller(),

      skip_dma_cycles: false,
    }
  }

  pub fn save_binary<'a>(&'a self, mut writer: BufWriter<&'a mut File>) -> BufWriter<&mut File> {
    into_writer(&self.ram, &mut writer).unwrap();
    into_writer(&self.ext_ram, &mut writer).unwrap();
    into_writer(&self.skip_dma_cycles, &mut writer).unwrap();
    let mapper = self.mapper.as_ref().unwrap().borrow();

    into_writer(&mapper.mapper_type(), &mut writer).unwrap();
    into_writer(&mapper.save(), &mut writer).unwrap();
    writer
  }

  pub fn load_binary(
    mut reader: BufReader<File>,
    message_sx: Sender<Message>,
    ppu: Arc<Mutex<Ppu>>,
    apu: Arc<Mutex<Apu>>,
  ) -> Self {
    let ram: Vec<Byte> = from_reader(&mut reader).unwrap();
    let ext_ram: Vec<Byte> = from_reader(&mut reader).unwrap();
    let skip_dma_cycles: bool = from_reader(&mut reader).unwrap();

    let mapper_type: u8 = from_reader(&mut reader).unwrap();
    let mapper_content: String = from_reader(&mut reader).unwrap();
    let ppu_clone = ppu.clone();
    let mapper = load_mapper(
      mapper_type as Byte,
      mapper_content.as_str(),
      Box::new(move |val: Byte| {
        let r = ppu_clone.try_lock();
        if r.is_err() {
          warn!("ppu is locked");
          return;
        }
        r.unwrap().update_mirroring(Some(val));
      }),
      Box::new(move || {
        if let Err(e) = message_sx.send(Message::CpuInterrupt(InterruptType::IRQ)) {
          log::error!("send interrupt error {:?}", e);
        }
      })
    );
    ppu.lock().unwrap().set_mapper_for_bus(mapper.clone());
    Self {
      ram,
      ext_ram,
      has_ext_ram: false,
      mapper: Some(mapper),
      registers: vec![ppu, apu],
      control1: Controller::new(),
      control2: Controller::new(),
      skip_dma_cycles,
    }
  }

  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    if mapper.borrow().has_extended_ram() {
      self.has_ext_ram = true;
      self.ext_ram.resize(0x2000, 0);
    }
    self.mapper = Some(mapper);
  }

  pub fn set_controller_keys(&mut self, p1: Vec<KeyType>, p2: Vec<KeyType>) {
    self.control1.set_key_bindings(p1);
    self.control2.set_key_bindings(p2);
  }

  pub fn check_and_reset_dma(&mut self) -> bool {
    let ret = self.skip_dma_cycles;
    self.skip_dma_cycles = false;
    ret
  }

  pub fn write(&mut self, addr: Address, value: Byte) {
    match addr {
      0x0000..=0x1fff => {
        self.ram[(addr & 0x07ff) as usize] = value;
      }
      0x2000..=0x401f => {
        let mapped_addr = if addr < 0x4000 {
          // PPU registers, mirrored
          addr & PPU_DATA
        } else {
          addr
        };
        match mapped_addr {
          JOY1 => {
            self.control1.strobe(value);
            self.control2.strobe(value);
          }
          OAM_DMA => {
            self.skip_dma_cycles = true;
            unsafe {
              let ptr = self.get_page_ptr(value);
              if let Some(ptr) = ptr {
                for reg in &mut self.registers {
                  if reg.lock().unwrap().dma(ptr) {
                    break;
                  }
                }
              }
            }
          }
          _ => {
            for reg in &mut self.registers {
              if reg.lock().unwrap().write(mapped_addr, value) {
                break;
              }
            }
          }
        }
      }
      0x4020..=0x5fff => {
        // self
        //   .mapper
        //   .as_ref()
        //   .unwrap()
        //   .borrow_mut()
        //   .write_prg(addr, value);
      }
      0x6000..=0x7fff => {
        if self.has_ext_ram {
          self.ext_ram[(addr - 0x6000) as usize] = value;
        }
      }
      0x8000..=0xffff => {
        self
          .mapper
          .as_ref()
          .unwrap()
          .borrow_mut()
          .write_prg(addr, value);
      }
    }
  }

  #[inline]
  pub fn save_read(&self, addr: Address) -> Byte {
    match addr {
      0x0000..=0x1fff => self.ram[(addr & 0x07ff) as usize],
      0x2000..=0x401f => 0,
      0x4020..=0x5fff => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
      0x6000..=0x7fff => {
        if self.has_ext_ram {
          self.ext_ram[(addr - 0x6000) as usize]
        } else {
          0
        }
      }
      _ => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
    }
  }

  pub fn read_extra(&mut self, addr: Address) -> Byte {
    let mapped_addr = if addr < 0x4000 {
      // PPU registers, mirrored
      addr & 0x2007
    } else {
      addr
    };
    return match mapped_addr {
      JOY1 => self.control1.read(),
      JOY2 => self.control2.read(),
      _ => {
        for reg in &mut self.registers {
          if let Some(value) = reg.lock().unwrap().read(mapped_addr) {
            return value;
          }
        }
        warn!("Attempt to read at {:#x} without callback registered", addr);
        0
      }
    };
  }

  #[inline]
  pub fn read(&mut self, addr: Address) -> Byte {
    if addr < 0x4020 && addr > 0x2000 {
      return self.read_extra(addr);
    }
    self.save_read(addr)
  }

  #[inline]
  pub fn read_addr(&mut self, addr: Address) -> Address {
    self.read(addr) as Address
  }

  pub unsafe fn get_page_ptr(&self, page: Byte) -> Option<*const Byte> {
    let addr = (page as usize) << 8;
    if addr < 0x2000 {
      let ptr = self.ram.as_ptr();
      Some(ptr.add(addr & 0x7FF))
    } else if addr < 0x4020 {
      error!("Attempting to access register address memory");
      None
    } else if addr < 0x6000 {
      error!("Not supported to access expansion ROM");
      None
    } else if addr < 0x8000 {
      let ptr = self.ext_ram.as_ptr();
      Some(ptr.add(addr - 0x6000))
    } else {
      None
    }
  }
}
