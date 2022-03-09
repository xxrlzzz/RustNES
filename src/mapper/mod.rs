pub mod cn_rom;
pub mod factory;
pub mod n_rom;
pub mod ux_rom;

use crate::types::*;

pub trait Mapper {
  fn write_prg(&mut self, addr: Address, value: Byte);
  fn read_prg(&self, addr: Address) -> Byte;
  fn write_chr(&mut self, addr: Address, value: Byte);
  fn read_chr(&self, addr: Address) -> Byte;

  fn has_extended_ram(&self) -> bool;

  fn get_name_table_mirroring(&self) -> u8;
}
