use log::warn;
use serde::{Deserialize, Serialize};

use crate::cartridge::{Cartridge, BANK_SIZE};
use crate::common::*;
use crate::mapper::Mapper;

use super::factory::NROM;
use super::save;

#[derive(Serialize, Deserialize)]
pub struct NRom {
  one_bank: bool,
  character_ram: Option<Vec<Byte>>,
  cart: Cartridge,
}

impl NRom {
  pub fn new(cart: Cartridge) -> Self {
    let ram = if cart.get_vrom().len() == 0 {
      Some(vec![0; 0x2000])
    } else {
      None
    };
    Self {
      one_bank: cart.get_rom().len() == BANK_SIZE,
      character_ram: ram,
      cart: cart,
    }
  }
}

impl Mapper for NRom {
  #[inline]
  fn read_prg(&self, addr: Address) -> Byte {
    if self.one_bank {
      self.cart.get_rom()[((addr - 0x8000) & 0x3FFF) as usize]
    } else {
      self.cart.get_rom()[(addr - 0x8000) as usize]
    }
  }

  #[inline]
  fn write_prg(&mut self, addr: Address, _: Byte) {
    warn!("ROM memory write attempt at {:#x}", addr);
  }

  #[inline]
  fn read_chr(&self, addr: Address) -> Byte {
    match &self.character_ram {
      Some(ram) => ram[addr as usize],
      None => self.cart.get_vrom()[addr as usize],
    }
  }

  #[inline]
  fn write_chr(&mut self, addr: Address, value: Byte) {
    match &mut self.character_ram {
      Some(ram) => ram[addr as usize] = value,
      None => warn!("Attempting to write read-only CHR memory on {:#x}", addr),
    }
  }

  #[inline]
  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  #[inline]
  fn get_name_table_mirroring(&self) -> u8 {
    self.cart.get_name_table_mirroring()
  }

  fn save(&self) -> String {
    save(self)
  }

  fn mapper_type(&self) -> u8 {
    NROM
  }
}

unsafe impl Send for NRom {}
unsafe impl Sync for NRom {}
