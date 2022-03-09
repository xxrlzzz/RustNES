use crate::cartridge::Cartridge;
use crate::mapper::Mapper;
use crate::types::*;
use log::warn;
use std::vec::Vec;

pub struct UxRom {
  ues_character_ram: bool,
  select_chr: Address,
  character_ram: Vec<Byte>,
  cart: Cartridge,
}

impl UxRom {
  pub fn new(cart: Cartridge) -> Self {
    let ues_character_ram = cart.get_vrom().len() == 0;
    let mut ret = Self {
      ues_character_ram: ues_character_ram,
      select_chr: 0,
      character_ram: vec![],
      cart: cart,
    };
    if ues_character_ram {
      ret.character_ram.reserve(0x2000);
    }
    ret
  }

  fn read_last_bank(&self, addr: Address) -> Byte {
    return self.cart.get_rom()[self.cart.get_rom().len() - 0x4000 + addr as usize];
  }
}

impl Mapper for UxRom {
  fn read_prg(&self, addr: Address) -> Byte {
    if addr < 0xC000 {
      self.cart.get_rom()[(((addr - 0x8000) & 0x3FFF) | (self.select_chr << 14)) as usize]
    } else {
      self.read_last_bank(addr & 0x3FFF)
    }
  }

  fn write_prg(&mut self, _: Address, value: Byte) {
    self.select_chr = value as Address;
  }

  fn read_chr(&self, addr: Address) -> Byte {
    if self.ues_character_ram {
      self.character_ram[addr as usize]
    } else {
      self.cart.get_vrom()[addr as usize]
    }
  }

  fn write_chr(&mut self, addr: Address, value: Byte) {
    if self.ues_character_ram {
      self.character_ram[addr as usize] = value;
    } else {
      warn!("Attempting to write read-only CHR memory on {:#x}", addr);
    }
  }

  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    self.cart.get_name_table_mirroring()
  }
}
