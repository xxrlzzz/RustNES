use rust_emu_common::{component::cartridge::Cartridge, mapper::Mapper, types::*};

use crate::cartridge::GBACartridge;


pub struct NRom {
  cart: GBACartridge,
  character_ram: [Byte; 0x2000],
}

impl NRom {
  pub fn new(cart: GBACartridge) -> Self {
    NRom { 
      cart,
      
      character_ram: [0; 0x2000],
    }
  }
}

impl Mapper for NRom {
    fn write_prg(&mut self, _: Address, _: Byte) {
      return;
    }
    
    fn read_prg(&self, addr: Address) -> Byte {
      self.cart.get_rom()[addr as usize]
    }

    fn write_chr(&mut self, addr: Address, value: Byte) {
      self.character_ram[addr as usize - 0x8000] = value;
    }

    fn read_chr(&self, addr: Address) -> Byte {
      self.character_ram[addr as usize - 0x8000]
    }

    fn has_extended_ram(&self) -> bool {
        todo!()
    }

    fn get_name_table_mirroring(&self) -> u8 {
        todo!()
    }

    fn save(&self) -> String {
        todo!()
    }

    fn mapper_type(&self) -> u8 {
        todo!()
    }
}