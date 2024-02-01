use rust_emu_common::types::*;

use crate::ppu::OamEntry;

pub(crate) struct PictureBus {
  oam_ram: [OamEntry; 40],
  vram: [Byte; 0x2000],
}

impl Default for PictureBus {
  fn default() -> Self {
    PictureBus::new()
  }
}

impl PictureBus {
  pub(crate) fn new() -> Self {
    PictureBus {
      oam_ram: [OamEntry::default(); 40],
      vram: [0; 0x2000],
    }
  }

  pub(crate) fn oam_ram(&self) -> &[OamEntry; 40] {
    &self.oam_ram
  }

  pub(crate) fn oam_write(&mut self, addr: Address, data: Byte) {
    let addr = if addr >= 0xFE00 { addr - 0xFE00 } else { addr };
    let idx = (addr / 4) as usize;
    let offset = addr % 4;
    if offset == 0 {
      self.oam_ram[idx].y = data;
    } else if offset == 1 {
      self.oam_ram[idx].x = data;
    } else if offset == 2 {
      self.oam_ram[idx].title = data;
    } else {
      self.oam_ram[idx].flag = data;
    }
  }

  pub(crate) fn oam_read(&self, addr: Address) -> Byte {
    let addr = if addr >= 0xFE00 { addr - 0xFE00 } else { addr };
    let idx = addr / 3;
    let offset = addr % 3;
    if offset == 0 {
      self.oam_ram[idx as usize].y
    } else if offset == 1 {
      self.oam_ram[idx as usize].x
    } else {
      self.oam_ram[idx as usize].title
    }
  }

  pub(crate) fn vram_write(&mut self, addr: Address, data: Byte) {
    self.vram[(addr - 0x8000) as usize] = data;
  }

  pub(crate) fn vram_read(&self, addr: Address) -> Byte {
    self.vram[(addr - 0x8000) as usize]
  }
}
