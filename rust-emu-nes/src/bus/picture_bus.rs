use log::info;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use rust_emu_common::types::*;

use crate::mapper::Mapper;

mod name_table_mirroring {
  pub const HORIZONTAL: u8 = 0;
  pub const VERTICAL: u8 = 1;
  pub const FOUR_SCREEN: u8 = 8;
  pub const ONE_SCREEN_LOWER: u8 = 9;
  pub const ONE_SCREEN_HIGHER: u8 = 10;
}

#[derive(Serialize, Deserialize)]
pub struct PictureBus {
  ram: Vec<Byte>,
  // Indices where they start in RAM vector.
  name_table0: usize,
  name_table1: usize,
  name_table2: usize,
  name_table3: usize,
  palette: Vec<Byte>,
  #[serde(skip)]
  mapper: Option<Rc<RefCell<dyn Mapper>>>, // TODO: move mapper to PPU to save lock time?
}

impl PictureBus {
  pub fn new() -> Self {
    Self {
      ram: vec![0; 0x800],
      name_table0: 0,
      name_table1: 0,
      name_table2: 0,
      name_table3: 0,
      palette: vec![0; 0x20],
      mapper: None,
    }
  }

  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    self.mapper = Some(mapper);
    self.update_mirroring(None);
  }

  #[inline]
  fn get_name_table(&self, addr: Address) -> usize {
    match addr {
      0x2000..=0x23FF => self.name_table0,
      0x2400..=0x27FF => self.name_table1,
      0x2800..=0x2BFF => self.name_table2,
      _ => self.name_table3,
    }
  }

  #[inline]
  pub fn batch_read(&self, addr1: Address, addr2: Address, shift_time: u8) -> Byte {
    let mapper = self.mapper.as_ref().unwrap().borrow();
    let value1 = match addr1 {
      0x0000..=0x1FFF => mapper.read_chr(addr1),
      0x2000..=0x3EFF => self.ram[self.get_name_table(addr1) + (addr1 & 0x3FF) as usize],
      0x3F00..=0x3FFF => self.palette[(addr1 & 0x1F) as usize],
      _ => 0,
    } >> shift_time;
    let value2 = match addr2 {
      0x0000..=0x1FFF => mapper.read_chr(addr2),
      0x2000..=0x3EFF => self.ram[self.get_name_table(addr2) + (addr2 & 0x3FF) as usize],
      0x3F00..=0x3FFF => self.palette[(addr2 & 0x1F) as usize],
      _ => 0,
    } >> shift_time;
    value1 & 1 | ((value2 & 1) << 1)
  }

  #[inline]
  pub fn read(&self, addr: Address) -> Byte {
    match addr {
      // TODO(xxrl) avoid borrow for each time reading will save performance.
      0x0000..=0x1FFF => self.mapper.as_ref().unwrap().borrow().read_chr(addr),
      0x2000..=0x3EFF => self.ram[self.get_name_table(addr) + (addr & 0x3FF) as usize],
      0x3F00..=0x3FFF => self.palette[(addr & 0x1F) as usize],
      _ => 0,
    }
  }

  #[inline]
  pub fn read_palette(&self, palette_addr: Byte) -> Byte {
    self.palette[palette_addr as usize]
  }

  pub fn write(&mut self, addr: Address, value: Byte) {
    if addr < 0x2000 {
      self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .write_chr(addr, value);
    } else if addr < 0x3EFF {
      let idx = self.get_name_table(addr) + (addr & 0x3FF) as usize;
      self.ram[idx] = value;
    } else if addr < 0x3FFF {
      if addr == 0x3F10 {
        self.palette[0] = value;
      } else {
        self.palette[(addr & 0x1F) as usize] = value;
      }
    }
  }

  pub fn update_mirroring(&mut self, mirror: Option<u8>) {
    let mirror = match mirror {
      Some(m) => m,
      None => self
        .mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .get_name_table_mirroring(),
    };
    match mirror {
      name_table_mirroring::HORIZONTAL => {
        self.name_table0 = 0;
        self.name_table1 = 0;
        self.name_table2 = 0x400;
        self.name_table3 = 0x400;
        info!("Horizontal Name Table mirroring set. (Vertical Scrolling)");
      }
      name_table_mirroring::VERTICAL => {
        self.name_table0 = 0;
        self.name_table1 = 0x400;
        self.name_table2 = 0;
        self.name_table3 = 0x400;
        info!("Vertical Name Table mirroring set. (Horizontal Scrolling)");
      }
      name_table_mirroring::ONE_SCREEN_LOWER => {
        self.name_table0 = 0;
        self.name_table1 = 0;
        self.name_table2 = 0;
        self.name_table3 = 0;
        info!("Single Screen mirroring set with lower bank.");
      }
      name_table_mirroring::ONE_SCREEN_HIGHER => {
        self.name_table0 = 0x400;
        self.name_table1 = 0x400;
        self.name_table2 = 0x400;
        self.name_table3 = 0x400;
        info!("Single Screen mirroring set with higher bank.");
      }
      name_table_mirroring::FOUR_SCREEN => {
        self.name_table0 = self.ram.len();
      }
      _ => {
        self.name_table0 = 0;
        self.name_table1 = 0;
        self.name_table2 = 0;
        self.name_table3 = 0;
        info!("Unsupported name table mirroring was set {}", mirror);
      }
    }
  }

  pub fn scanline_irq(&mut self) {
    self.mapper.as_mut().unwrap().borrow_mut().scanline_irq()
  }
}
