use log::warn;
use serde::{Deserialize, Serialize};

use crate::cartridge::{Cartridge, BANK_SIZE};
use crate::common::*;
use crate::mapper::Mapper;

use super::factory::CNROM;
use super::save;

#[derive(Serialize, Deserialize)]
pub struct CnRom {
  one_bank: bool,
  select_chr: Address,
  cart: Cartridge,
}

impl CnRom {
  pub fn new(cart: Cartridge) -> Self {
    Self {
      one_bank: (cart.get_rom().len() == BANK_SIZE),
      select_chr: 0,
      cart: cart,
    }
  }
}

impl Mapper for CnRom {
  fn read_prg(&self, addr: Address) -> Byte {
    let target_addr = if !self.one_bank {
      addr - 0x8000
    } else {
      (addr - 0x8000) & 0x3FFF
    };
    self.cart.get_rom()[target_addr as usize]
  }

  fn write_prg(&mut self, _: Address, value: Byte) {
    self.select_chr = (value & 0x3) as u16;
  }

  fn read_chr(&self, addr: Address) -> Byte {
    self.cart.get_vrom()[(addr | (self.select_chr << 13)) as usize]
  }

  fn write_chr(&mut self, addr: Address, _: Byte) {
    warn!("Attempting to write read-only CHR memory on {:#x}", addr);
  }

  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    self.cart.get_name_table_mirroring()
  }

  fn save(&self) -> String {
    save(self)
  }

  fn mapper_type(&self) -> u8 {
    CNROM
  }
}

unsafe impl Sync for CnRom {}
unsafe impl Send for CnRom {}
