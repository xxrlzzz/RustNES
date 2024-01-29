use rust_emu_common::types::*;
use serde::{Serialize, Deserialize};

use crate::cartridge::GBACartridge;


#[derive(Default, Serialize, Deserialize)]
pub struct MainBus {
  #[serde(with = "serde_bytes")]
  ram: Vec<Byte>,
  #[serde(with = "serde_bytes")]
  ext_ram: Vec<Byte>,
  #[serde(skip)]
  cart: GBACartridge,

  #[serde(skip)]
  has_ext_ram: bool,
}

impl MainBus {
    /**
     * 
     * 0x0000 - 0x3FFF : ROM Bank 0
     * 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
     * 0x8000 - 0x97FF : CHR RAM
     * 0x9800 - 0x9BFF : BG Map 1
     * 0x9C00 - 0x9FFF : BG Map 2
     * 0xA000 - 0xBFFF : Cartridge RAM
     * 0xC000 - 0xCFFF : RAM Bank 0
     * 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
     * 0xE000 - 0xFDFF : Reserved - Echo RAM
     * 0xFE00 - 0xFE9F : Object Attribute Memory
     * 0xFEA0 - 0xFEFF : Reserved - Unusable
     * 0xFF00 - 0xFF7F : I/O Registers
     * 0xFF80 - 0xFFFE : Zero Page
     */
    pub fn read(&self, addr: Address) -> Byte {
        match addr {
            0x0000..=0x7FFF => self.ram[addr as usize],
            0xA000..=0xBFFF => self.ext_ram[addr as usize - 0xA000],
            0xC000..=0xDFFF => self.ram[addr as usize - 0x4000],
            0xE000..=0xFDFF => self.ram[addr as usize - 0x2000],
            0xFE00..=0xFE9F => self.ram[addr as usize - 0x2000],
            0xFEA0..=0xFEFF => self.ram[addr as usize - 0x2000],
            0xFF00..=0xFF7F => self.ram[addr as usize - 0x2000],
            0xFF80..=0xFFFE => self.ram[addr as usize - 0x2000],
            0xFFFF => 0xFF,
            _ => panic!("Invalid address: {:#X}", addr),
        }
    }
}
