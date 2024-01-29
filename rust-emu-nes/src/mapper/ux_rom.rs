use log::warn;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

use crate::cartridge::NESCartridge;
use crate::mapper::Mapper;

use super::{save, UXROM};

use rust_emu_common::{component::cartridge::Cartridge, types::*};

#[derive(Serialize, Deserialize)]
pub struct UxRom {
  select_chr: Address,
  character_ram: Option<Vec<Byte>>,
  cart: NESCartridge,
}

impl UxRom {
  pub fn new(cart: NESCartridge) -> Self {
    let ram = if cart.get_vrom().len() == 0 {
      Some(vec![0; 0x2000])
    } else {
      None
    };
    Self {
      select_chr: 0,
      character_ram: ram,
      cart,
    }
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
    match &self.character_ram {
      Some(ram) => ram[addr as usize],
      None => self.cart.get_vrom()[addr as usize],
    }
  }

  fn write_chr(&mut self, addr: Address, value: Byte) {
    match &mut self.character_ram {
      Some(ram) => ram[addr as usize] = value,
      None => warn!("Attempting to write read-only CHR memory on {:#x}", addr),
    }
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
    UXROM
  }
}

unsafe impl Sync for UxRom {}
unsafe impl Send for UxRom {}
