use log::{error, warn};
use serde::{Deserialize, Serialize};
use serde_bytes::Bytes;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::rc::Rc;

use crate::apu::Apu;
use crate::common::types::*;
use crate::controller::key_binding_parser::KeyType;
use crate::controller::Controller;
use crate::mapper::factory::load_mapper;
use crate::mapper::Mapper;
use crate::ppu::Ppu;

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

#[derive(Default, Serialize, Deserialize)]
pub struct MainBus {
  #[serde(with = "serde_bytes")]
  ram: Vec<Byte>,
  #[serde(with = "serde_bytes")]
  ext_ram: Vec<Byte>,
  #[serde(skip)]
  mapper: Option<Rc<RefCell<dyn Mapper>>>,
  #[serde(skip)]
  registers: Vec<Rc<RefCell<dyn RegisterHandler>>>,
  #[serde(skip)]
  control1: Controller,
  #[serde(skip)]
  control2: Controller,

  skip_dma_cycles: bool,
}

impl MainBus {
  pub fn new(apu: Rc<RefCell<Apu>>, ppu: Rc<RefCell<Ppu>>) -> Self {
    Self {
      ram: vec![0; 0x800],
      ext_ram: vec![],
      mapper: None,
      registers: vec![ppu, apu],
      control1: Controller::new(),
      control2: Controller::new(),

      skip_dma_cycles: false,
    }
  }

  pub fn save(&self) -> Value {
    json!({
      "ram": serde_json::to_string(Bytes::new(&self.ram)).unwrap(),
      "ext_ram":serde_json::to_string(Bytes::new(&self.ext_ram)).unwrap(),
      "mapper" : self.mapper.as_ref().map(|m| m.borrow().save()).unwrap_or(String::new()),
      "skip_dma_cycles": self.skip_dma_cycles,
      "mapper_type": self.mapper.as_ref().unwrap().borrow().mapper_type(),
    })
  }

  pub fn load(json: &serde_json::Value, ppu: Rc<RefCell<Ppu>>, apu: Rc<RefCell<Apu>>) -> Self {
    let mapper_type = json.get("mapper_type").unwrap().as_u64().unwrap();
    let mapper_content = json.get("mapper").unwrap().as_str().unwrap();
    let mapper = load_mapper(mapper_type as Byte, mapper_content);
    ppu.borrow_mut().set_mapper_for_bus(mapper.clone());
    Self {
      ram: serde_json::from_str(json.get("ram").unwrap().as_str().unwrap()).unwrap(),
      ext_ram: serde_json::from_str(json.get("ext_ram").unwrap().as_str().unwrap()).unwrap(),
      mapper: Some(mapper),
      registers: vec![ppu, apu],
      control1: Controller::new(),
      control2: Controller::new(),
      skip_dma_cycles: json.get("skip_dma_cycles").unwrap().as_bool().unwrap(),
    }
  }

  #[cfg(feature = "use_gl")]
  pub fn set_window(&mut self, window: Rc<RefCell<glfw::Window>>) {
    self.control1.set_window(window.clone());
    self.control2.set_window(window)
  }

  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    self.mapper = Some(mapper);
    if self.mapper.as_ref().unwrap().borrow().has_extended_ram() {
      self.ext_ram.resize(0x2000, 0);
    }
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
                if reg.borrow_mut().dma(ptr) {
                  break;
                }
              }
            }
          }
        }
        _ => {
          for reg in &mut self.registers {
            if reg.borrow_mut().write(mapped_addr, value) {
              break;
            }
          }
        }
      }
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
        JOY1 => self.control1.read(),
        JOY2 => self.control2.read(),
        _ => {
          for reg in &mut self.registers {
            if let Some(value) = reg.borrow_mut().read(mapped_addr) {
              return value;
            }
          }
          warn!("Attempt to read at {:#x} without callback registered", addr);
          return 0;
        }
      };
    }
    if addr < 0x6000 {
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
