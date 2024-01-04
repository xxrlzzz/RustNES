use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::common::{bit_eq, Byte};
use crate::{cartridge::Cartridge, common::Address};

use super::{SXROM, save};

use super::{
  factory::{MirrorCallback, NameTableMirroring},
  Mapper,
};

const BANK_SIZE: usize = 0x1000;

#[derive(Serialize, Deserialize)]
pub struct SxRom {
  character_ram: Option<Vec<Byte>>,
  cart: Cartridge,
  #[serde(skip)]
  mirror_cb: Option<MirrorCallback>,
  mirroring: NameTableMirroring,

  mode_chr: Byte,
  mode_prg: Byte,

  temp_register: Byte,
  write_counter: i32,

  reg_prg: Byte,
  reg_chr0: Byte,
  reg_chr1: Byte,

  first_bank_prg: usize,  // offset of rom
  second_bank_prg: usize, // offset of rom

  first_bank_chr: usize,  // offset of vrom
  second_bank_chr: usize, // offset of vrom
}

impl SxRom {
  pub fn new(cart: Cartridge, mirror_cb: MirrorCallback) -> Self {
    let ram = if cart.get_vrom().len() == 0 {
      Some(vec![0; 0x2000])
    } else {
      None
    };
    let rom_len = cart.get_rom().len();
    Self {
      character_ram: ram,
      cart,
      mirror_cb: Some(mirror_cb),
      mirroring: NameTableMirroring::Horizontal,
      mode_chr: 0,
      mode_prg: 3,
      temp_register: 0,
      write_counter: 0,
      reg_prg: 0,
      reg_chr0: 0,
      reg_chr1: 0,

      first_bank_prg: 0,
      second_bank_prg: rom_len - 0x4000, // 0x2000 * 0x0e last bank,

      first_bank_chr: 0,
      second_bank_chr: 0,
    }
  }
}

impl SxRom {
  fn calculate_prg_pointers(&mut self) {
    if self.mode_prg <= 1 {
      // 32KB changeable
      self.first_bank_prg = 0x8000 * (self.reg_prg >> 1) as usize;
      self.second_bank_prg = self.first_bank_prg + 0x4000; // add 16KB
    } else if self.mode_prg == 2 {
      // fir first switch second fixed
      self.first_bank_prg = 0;
      self.second_bank_prg = 0x4000 * self.reg_prg as usize;
    } else {
      // switch first fix second
      self.first_bank_prg = 0x4000 * self.reg_prg as usize;
      self.second_bank_prg = self.cart.get_rom().len() - 0x4000;
    }
  }

  pub fn set_mirror_cb(&mut self, mirror_cb: MirrorCallback) {
    self.mirror_cb = Some(mirror_cb);
  }
}

impl Mapper for SxRom {
  fn write_prg(&mut self, addr: Address, value: Byte) {
    if !bit_eq(value, 0x80) {
      // reset bit not set.
      self.temp_register = (self.temp_register >> 1) | ((value & 1) << 4);
      self.write_counter += 1;

      if self.write_counter == 5 {
        if addr <= 0x9fff {
          match self.temp_register & 0x3 {
            0 => self.mirroring = NameTableMirroring::OneScreenLower,
            1 => self.mirroring = NameTableMirroring::OneScreenHigher,
            2 => self.mirroring = NameTableMirroring::Vertical,
            3 => self.mirroring = NameTableMirroring::Horizontal,
            _ => unreachable!(),
          }
          if self.mirror_cb.is_some() {
            self.mirror_cb.as_mut().unwrap()(self.mirroring.into());
          }

          self.mode_chr = (self.temp_register & 0x10) >> 4;
          self.mode_prg = (self.temp_register & 0xc) >> 2;
          self.calculate_prg_pointers();

          // Recalculate CHR pointers
          if self.mode_chr == 0 {
            // one 8KB bank
            self.first_bank_chr = BANK_SIZE * (self.reg_chr0 | 1) as usize;
            self.second_bank_chr = self.first_bank_chr + BANK_SIZE;
          } else {
            // two 4KB banks
            self.first_bank_chr = BANK_SIZE * self.reg_chr0 as usize;
            self.second_bank_chr = BANK_SIZE * self.reg_chr1 as usize;
          }
        } else if addr <= 0xbfff {
          // CHR Reg 0
          self.reg_chr0 = self.temp_register;
          self.first_bank_prg = BANK_SIZE * (self.temp_register | (1 - self.mode_chr)) as usize;
          if self.mode_chr == 0 {
            self.second_bank_chr = self.first_bank_chr + BANK_SIZE;
          }
        } else if addr <= 0xdfff {
          self.reg_chr1 = self.temp_register;
          if self.mode_chr == 1 {
            self.second_bank_chr = BANK_SIZE * self.temp_register as usize;
          }
        } else {
          // TODO PRG-RAM
          if bit_eq(self.temp_register, 0x10) {
            info!("PRG-RAM activated");
          }

          self.temp_register &= 0xf;
          self.reg_prg = self.temp_register;
          self.calculate_prg_pointers();
        }

        self.temp_register = 0;
        self.write_counter = 0;
      }
    } else {
      // reset
      self.temp_register = 0;
      self.write_counter = 0;
      self.mode_prg = 3;
      self.calculate_prg_pointers();
    }
  }

  fn read_prg(&self, addr: Address) -> Byte {
    self.cart.get_rom()[if addr < 0xC000 {
      self.first_bank_prg
    } else {
      self.second_bank_prg
    } + (addr & 0x3FFF) as usize]
  }

  fn write_chr(&mut self, addr: Address, value: Byte) {
    match &mut self.character_ram {
      Some(ram) => ram[addr as usize] = value,
      None => warn!("Attempting to write read-only CHR memory on {:#x}", addr),
    }
  }

  fn read_chr(&self, addr: Address) -> Byte {
    match &self.character_ram {
      Some(ram) => ram[addr as usize],
      None => self.cart.get_vrom()[if addr < BANK_SIZE as Address {
        self.first_bank_chr + addr as usize
      } else {
        self.second_bank_chr + (addr & 0xfff) as usize
      }],
    }
  }

  fn has_extended_ram(&self) -> bool {
    self.cart.has_extended_ram()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    self.mirroring.into()
  }

  fn save(&self) -> String {
    save(self)
  }

  fn mapper_type(&self) -> u8 {
    SXROM
  }
}
