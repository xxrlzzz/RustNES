use crate::{
  cartridge::Cartridge,
  common::{bit_eq, Address, Byte},
};

use super::{
  factory::{MirrorCallback, NameTableMirroring, IRQCallback},
  Mapper, TXROM,
};
use serde::{Deserialize, Serialize};

/**
 * MMC3 mapper
 */
#[derive(Serialize, Deserialize)]
pub struct TxRom {
  cart: Cartridge,
  target_register: usize,
  prg_bank_mode: bool,
  chr_inversion: bool,

  bank_register: [Address; 8],

  irq_enable: bool,
  irq_counter: u32,
  irq_latch: u8,
  irq_reload_pending: bool,

  prg_ram: Vec<Byte>,
  mirroring_ram: Vec<Byte>,

  prg_bank0: usize,
  prg_bank1: usize,
  prg_bank2: usize,
  prg_bank3: usize,

  chr_banks: [usize; 8],

  rom_size: usize,
  cart_mirroring: u8,

  mirroring: NameTableMirroring,
  #[serde(skip)]
  // mirroring callback
  mirror_cb: Option<MirrorCallback>,
  #[serde(skip)]
  // interrupt callback
  interrupt_cb: Option<IRQCallback>,
}

impl TxRom {
  pub fn new(cart: Cartridge, mirror_cb: MirrorCallback, interrupt_cb: IRQCallback) -> Self {
    let rom_len = cart.get_rom().len();
    let vrom_len = std::cmp::max(cart.get_vrom().len(), 0x800);
    let cart_mirroring = cart.get_name_table_mirroring();
    let mut ret = TxRom {
      cart,
      target_register: 0,
      prg_bank_mode: false,
      chr_inversion: false,
      bank_register: [0; 8],
      irq_enable: false,
      irq_counter: 0,
      irq_latch: 0,
      irq_reload_pending: false,
      prg_ram: vec![0; 32 * 1024],
      rom_size: rom_len as usize,
      cart_mirroring: cart_mirroring,
      mirroring_ram: vec![0; 4 * 1024],
      prg_bank0: rom_len - 0x4000,
      prg_bank1: rom_len - 0x2000,
      prg_bank2: rom_len - 0x4000,
      prg_bank3: rom_len - 0x2000,
      chr_banks: [vrom_len - 0x400; 8],
      mirroring: NameTableMirroring::Horizontal,
      mirror_cb: Some(mirror_cb),
      interrupt_cb: Some(interrupt_cb)
    };
    ret.chr_banks[0] = vrom_len - 0x800;
    ret.chr_banks[3] = vrom_len - 0x800;

    ret
  }

  pub fn set_mirror_cb(&mut self, mirror_cb: MirrorCallback) {
    self.mirror_cb = Some(mirror_cb);
  }

  pub fn set_irq_cb(&mut self, irq_cb: IRQCallback) {
    self.interrupt_cb = Some(irq_cb);
  }

  #[inline]
  fn read_prg_bank(&self, addr: usize) -> Byte {
    self.cart.get_rom()[addr]
  }

  fn update_bank_offset(&self, index: Address) -> usize {
    if self.cart.get_vrom().len() == 0 {
      return 0
    }
    let truncate = (index as usize) % (self.cart.get_vrom().len() / 0x400);
    truncate * 0x400
  }
}

impl Mapper for TxRom {
  fn write_prg(&mut self, addr: Address, value: Byte) {
    match addr {
      0x6000..=0x7fff => {
        self.prg_ram[(addr & 0x1fff) as usize] = value;
      }
      0x8000..=0x9fff => {
        if !(bit_eq(addr, 0x01)) {
          self.target_register = (value & 0x7) as usize;
          self.prg_bank_mode = bit_eq(value, 0x40);
          self.chr_inversion = bit_eq(value, 0x80);
        } else {
          self.bank_register[self.target_register] = value as Address;

          if !self.chr_inversion {
            // Add 0xfe mask to ignore lowest bit
            self.chr_banks[0] = self.update_bank_offset(self.bank_register[0] & 0xFE);
            self.chr_banks[1] = self.update_bank_offset(self.bank_register[0] & 0xFE + 1);
            self.chr_banks[2] = self.update_bank_offset(self.bank_register[1] & 0xFE);
            self.chr_banks[3] = self.update_bank_offset(self.bank_register[1] & 0xFE + 1);
            self.chr_banks[4] = self.update_bank_offset(self.bank_register[2]);
            self.chr_banks[5] = self.update_bank_offset(self.bank_register[3]);
            self.chr_banks[6] = self.update_bank_offset(self.bank_register[4]);
            self.chr_banks[7] = self.update_bank_offset(self.bank_register[5]);
          } else {
            self.chr_banks[0] = self.update_bank_offset(self.bank_register[2]);
            self.chr_banks[1] = self.update_bank_offset(self.bank_register[3]);
            self.chr_banks[2] = self.update_bank_offset(self.bank_register[4]);
            self.chr_banks[3] = self.update_bank_offset(self.bank_register[5]);
            self.chr_banks[4] = self.update_bank_offset(self.bank_register[0] & 0xFE);
            self.chr_banks[5] = self.update_bank_offset(self.bank_register[0] & 0xFE + 1);
            self.chr_banks[6] = self.update_bank_offset(self.bank_register[1] & 0xFE);
            self.chr_banks[7] = self.update_bank_offset(self.bank_register[1] & 0xFE + 1);
          }

          if !self.prg_bank_mode {
            // ignore top two bits for R6 / R7 using 0x3F
            self.prg_bank0 = (self.bank_register[6] & 0x3f) as usize * 0x2000;
            self.prg_bank1 = (self.bank_register[7] & 0x3f) as usize * 0x2000;
            self.prg_bank2 = self.rom_size - 0x4000;
            self.prg_bank3 = self.rom_size - 0x2000;
          } else {
            self.prg_bank0 = self.rom_size - 0x4000;
            self.prg_bank1 = (self.bank_register[7] & 0x3f) as usize * 0x2000;
            self.prg_bank2 = (self.bank_register[6] & 0x3f) as usize * 0x2000;
            self.prg_bank3 = self.rom_size - 0x2000;
          }
        }
      }
      0xa000..=0xbfff => {
        // self.prg_bank1 = value as usize;
        if !bit_eq(addr, 0x01) {
          // Mirroring
          if bit_eq(self.cart_mirroring, 0x8) {
            self.mirroring = NameTableMirroring::FourScreen
          } else if bit_eq(value, 0x01) {
            self.mirroring = NameTableMirroring::Horizontal
          } else {
            self.mirroring = NameTableMirroring::Vertical
          }
          self.mirror_cb.as_mut().unwrap()(self.mirroring.into());
        } else {
          // PRG Ram Protect
        }
      }
      0xc000..=0xdfff => {
        if !(bit_eq(addr, 0x01)) {
          self.irq_latch = value;
        } else {
          self.irq_counter = 0;
          self.irq_reload_pending = true;
        }
      }
      0xe000.. => {
        // enable if add address
        self.irq_enable = bit_eq(addr & 0x01, 0x01)
        // TODO acknowledge any pending interrupts?
      }
      _ => {}
    }
  }

  fn read_prg(&self, addr: Address) -> Byte {
    match addr {
      0x6000..=0x7fff => self.prg_ram[(addr & 0x1fff) as usize],
      0x8000..=0x9fff => self.read_prg_bank(self.prg_bank0 + (addr & 0x1fff) as usize),
      0xa000..=0xbfff => self.read_prg_bank(self.prg_bank1 + (addr & 0x1fff) as usize),
      0xc000..=0xdfff => self.read_prg_bank(self.prg_bank2 + (addr & 0x1fff) as usize),
      0xe000..=0xffff => self.read_prg_bank(self.prg_bank3 + (addr & 0x1fff) as usize),
      _ => 0,
    }
  }

  fn write_chr(&mut self, addr: Address, value: Byte) {
    if addr >= 0x2000 && addr <= 0x2fff {
      self.mirroring_ram[(addr - 0x2000) as usize] = value;
    }
  }

  fn read_chr(&self, addr: Address) -> Byte {
    if addr <= 0x1fff {
      if self.cart.get_vrom().len() == 0 {
        return 0
      }
      let back_select = addr >> 10;
      let base_address = self.chr_banks[back_select as usize];
      self.cart.get_vrom()[(base_address + (addr & 0x3ff) as usize) as usize]
    } else if addr <= 0x2fff {
      self.mirroring_ram[(addr - 0x2000) as usize]
    } else {
      0
    }
  }

  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    self.mirroring.into()
  }

  fn save(&self) -> String {
    super::save(self)
  }

  fn mapper_type(&self) -> u8 {
    TXROM
  }

  fn scanline_irq(&mut self) {
      if self.irq_counter == 0 || self.irq_reload_pending {
        self.irq_counter = self.irq_latch as u32;
        self.irq_reload_pending = false;
      } else {
        self.irq_counter-=1;
        if self.irq_counter == 0 && self.irq_enable {
            self.interrupt_cb.as_mut().unwrap()();
        }
      }
  }
}
