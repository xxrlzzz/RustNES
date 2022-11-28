use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use crate::common::*;
use crate::mapper::Mapper;

mod name_table_mirroring {
  pub const HORIZONTAL: u8 = 0;
  pub const VERTICAL: u8 = 1;
  // pub const FOUR_SCREEN: u8 = 8;
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
  mapper: Option<Arc<Mutex<dyn Mapper + Send + Sync>>>,
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

  pub fn set_mapper(&mut self, mapper: Arc<Mutex<dyn Mapper + Send + Sync>>) {
    self.mapper = Some(mapper);
    self.update_mirroring();
  }

  fn get_name_table(&self, addr: Address) -> usize {
    if addr < 0x2400 {
      self.name_table0
    } else if addr < 0x2800 {
      self.name_table1
    } else if addr < 0x2C00 {
      self.name_table2
    } else {
      self.name_table3
    }
  }

  pub fn read(&self, addr: Address) -> Byte {
    if addr < 0x2000 {
      // TODO(xxrl) avoid borrow for each time reading will save performance.
      self.mapper.as_ref().unwrap().lock().unwrap().read_chr(addr)
    } else if addr < 0x3EFF {
      self.ram[self.get_name_table(addr) + (addr & 0x3FF) as usize]
    } else if addr < 0x3FFF {
      self.palette[(addr & 0x1F) as usize]
    } else {
      0
    }
  }

  pub fn read_palette(&self, palette_addr: Byte) -> Byte {
    self.palette[palette_addr as usize]
  }

  pub fn write(&mut self, addr: Address, value: Byte) {
    if addr < 0x2000 {
      self
        .mapper
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
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

  pub fn update_mirroring(&mut self) {
    match self
      .mapper
      .as_ref()
      .unwrap()
      .lock()
      .unwrap()
      .get_name_table_mirroring()
    {
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
      _ => {
        self.name_table0 = 0;
        self.name_table1 = 0;
        self.name_table2 = 0;
        self.name_table3 = 0;
        info!(
          "Unsupported name table mirroring was set {}",
          self
            .mapper
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .get_name_table_mirroring()
        );
      }
    }
  }
}
