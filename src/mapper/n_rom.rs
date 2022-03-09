use log::warn;

use crate::cartridge::{Cartridge, BANK_SIZE};
use crate::common::types::*;
use crate::mapper::Mapper;

pub struct NRom {
  one_bank: bool,
  use_character_ram: bool,
  character_ram: Vec<Byte>,
  cart: Cartridge,
}

impl NRom {
  pub fn new(cart: Cartridge) -> Self {
    let ues_character_ram = cart.get_vrom().len() == 0;
    let character_ram = if ues_character_ram {
      vec![0; 0x2000]
    } else {
      vec![0]
    };
    Self {
      one_bank: cart.get_rom().len() == BANK_SIZE,
      use_character_ram: ues_character_ram,
      character_ram: character_ram,
      cart: cart,
    }
  }
}

impl Mapper for NRom {
  fn read_prg(&self, addr: Address) -> Byte {
    if self.one_bank {
      self.cart.get_rom()[((addr - 0x8000) & 0x3FFF) as usize]
    } else {
      self.cart.get_rom()[(addr - 0x8000) as usize]
    }
  }

  fn write_prg(&mut self, addr: Address, _: Byte) {
    warn!("ROM memory write attempt at {:#x}", addr);
  }

  fn read_chr(&self, addr: Address) -> Byte {
    if self.use_character_ram {
      self.character_ram[addr as usize]
    } else {
      self.cart.get_vrom()[addr as usize]
    }
  }

  fn write_chr(&mut self, addr: Address, value: Byte) {
    if self.use_character_ram {
      self.character_ram[addr as usize] = value;
    } else {
      warn!("Attempting to write Read-only CHR memory at {:#x}", addr);
    }
  }

  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    self.cart.get_name_table_mirroring()
  }
}
