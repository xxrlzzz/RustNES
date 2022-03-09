use log::{error, warn};
use sfml::window::Key;
use std::cell::RefCell;
use std::rc::Rc;

use crate::common::types::*;
use crate::controller::Controller;
use crate::mapper::Mapper;
use crate::ppu::Ppu;

type IORegister = u16;

pub const PPU_CTRL: IORegister = 0x2000;
pub const PPU_MASK: IORegister = 0x2001;
pub const PPU_STATUS: IORegister = 0x2002;
pub const OAM_ADDR: IORegister = 0x2003;
pub const OAM_DATA: IORegister = 0x2004;
pub const PPU_SCROL: IORegister = 0x2005;
pub const PPU_ADDR: IORegister = 0x2006;
pub const PPU_DATA: IORegister = 0x2007;
pub const OAM_DMA: IORegister = 0x4014;
pub const APU_ADDR: IORegister = 0x4015; // Add.
pub const JOY1: IORegister = 0x4016;
pub const JOY2: IORegister = 0x4017;

pub struct MainBus {
  ram: Vec<Byte>,
  ext_ram: Vec<Byte>,
  mapper: Option<Rc<RefCell<dyn Mapper>>>,
  ppu: Rc<RefCell<Ppu>>,
  control1: Controller,
  control2: Controller,

  skip_dma_cycles: bool,
}

impl MainBus {
  pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
    Self {
      ram: vec![0; 0x800],
      ext_ram: vec![],
      mapper: None,
      ppu: ppu,
      control1: Controller::new(),
      control2: Controller::new(),

      skip_dma_cycles: false,
    }
  }

  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    self.mapper = Some(mapper);
    if self.mapper.as_ref().unwrap().borrow().has_extended_ram() {
      self.ext_ram.resize(0x2000, 0);
    }
  }

  pub fn set_controller_keys(&mut self, p1: Vec<Key>, p2: Vec<Key>) {
    self.control1.set_key_bindings(p1);
    self.control2.set_key_bindings(p2);
  }

  pub fn check_and_reset_dma(&mut self) -> bool {
    let ret = self.skip_dma_cycles;
    self.skip_dma_cycles = false;
    ret
  }

  pub fn write(&mut self, addr: Address, value: Byte) {
    if addr < 0x2000 {
      self.ram[(addr & 0x07ff) as usize] = value;
    } else if addr < 0x4020 {
      let mapped_addr = if addr < 0x4000 {
        // PPU registers, mirrored
        addr & PPU_DATA
      } else {
        addr
      };
      match mapped_addr {
        PPU_CTRL => self.ppu.borrow_mut().control(value),
        PPU_MASK => self.ppu.borrow_mut().set_mask(value),
        PPU_ADDR => self.ppu.borrow_mut().set_data_address(value),
        OAM_ADDR => self.ppu.borrow_mut().set_oam_address(value),
        PPU_SCROL => self.ppu.borrow_mut().set_scroll(value),
        PPU_DATA => self.ppu.borrow_mut().set_data(value),
        OAM_DATA => self.ppu.borrow_mut().set_oam_data(value),
        JOY1 => {
          self.control1.strobe(value);
          self.control2.strobe(value);
        }
        OAM_DMA => {
          self.skip_dma_cycles = true;
          unsafe {
            let ptr = self.get_page_ptr(value);
            if let Some(ptr) = ptr {
              self.ppu.borrow_mut().do_dma(ptr);
            }
          }
        }
        _ => {}
      };
    } else if addr < 0x6000 {
      warn!("Expansion ROM write attempted. This currently unsupported");
    } else if addr < 0x8000 {
      if self.mapper.as_ref().unwrap().borrow().has_extended_ram() {
        self.ext_ram[(addr - 0x6000) as usize] = value;
      }
    } else {
      self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .write_prg(addr, value);
    }
  }

  pub fn read(&mut self, addr: Address) -> Byte {
    if addr < 0x2000 {
      return self.ram[(addr & 0x7ff) as usize];
    }
    if addr < 0x4020 {
      let mapped_addr = if addr < 0x4000 {
        // PPU registers, mirrored
        addr & 0x2007
      } else {
        addr
      };
      return match mapped_addr {
        PPU_STATUS => self.ppu.borrow_mut().get_status(),
        PPU_DATA => self.ppu.borrow_mut().get_data(),
        OAM_ADDR => self.ppu.borrow().get_oam_data(),
        JOY1 => self.control1.read(),
        JOY2 => self.control2.read(),
        _ => {
          warn!("Attempt to read at {:#x} without callback registered", addr);
          0
        }
      };
    } else if addr < 0x6000 {
      warn!("Expansion ROM read attempted. This currently unsupported");
    } else if addr < 0x8000 {
      if self.mapper.as_ref().unwrap().borrow().has_extended_ram() {
        return self.ext_ram[(addr - 0x6000) as usize];
      }
    } else {
      return self.mapper.as_ref().unwrap().borrow().read_prg(addr);
    }
    0
  }

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
