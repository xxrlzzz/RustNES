use std::{cell::RefCell, rc::Rc};

use rust_emu_common::{mapper::Mapper, types::*};

use crate::ppu::OamEntry;

// #[derive(Serialize, Deserialize)]
pub(crate) struct PictureBus {
  oam_ram: [OamEntry; 40],
  // #[serde(skip)]
  mapper: Option<Rc<RefCell<dyn Mapper>>>,
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
      mapper: None,
    }
  }
  pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
      self.mapper = Some(mapper);
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

  // pub(crate) fn vram_write(&mut self, addr: Address, data: Byte) {
  //   self.mapper.as_mut().unwrap().borrow_mut().write_chr(addr, data);
  // }

  pub(crate) fn vram_read(&self, addr: Address) -> Byte {
    self.mapper.as_ref().unwrap().borrow().read_chr(addr)
  }

  pub(crate) fn read(&self, addr: Address) -> Byte {
    return match addr {
       // ROM data
       0x0000..=0x7FFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
       // Char/Map data
       0x8000..=0x9FFF => self.mapper.as_ref().unwrap().borrow().read_chr(addr),
       // Cartridge RAM
       0xA000..=0xBFFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
       _ => {log::warn!("unsupport dma read {}", addr); 0xFF}
    }
  }
}
