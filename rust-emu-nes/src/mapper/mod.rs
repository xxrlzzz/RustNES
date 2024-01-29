use rust_emu_common::types::*;

pub mod cn_rom;
pub mod factory;
pub mod n_rom;
pub mod ux_rom;
pub mod sx_rom;
pub mod tx_rom;

use serde::Serialize;


type MapperType = u8;
pub(crate) const NROM: MapperType = 0;
pub(crate) const SXROM: MapperType = 1;
pub(crate) const UXROM: MapperType = 2;
pub(crate) const CNROM: MapperType = 3;
pub(crate) const TXROM: MapperType = 4;
// TODO
pub(crate) const EXROM: MapperType = 5;
pub(crate) const AXROM: MapperType = 7;
pub(crate) const PXROM: MapperType = 9;

pub trait Mapper {
  fn write_prg(&mut self, addr: Address, value: Byte);
  fn read_prg(&self, addr: Address) -> Byte;
  fn write_chr(&mut self, addr: Address, value: Byte);
  fn read_chr(&self, addr: Address) -> Byte;

  fn has_extended_ram(&self) -> bool;

  fn scanline_irq(&mut self) {}

  fn get_name_table_mirroring(&self) -> u8;

  fn save(&self) -> String;

  fn mapper_type(&self) -> u8;
}

fn save<T>(t: &T) -> String
where
  T: Serialize + Mapper,
{
  serde_json::to_string(&t).unwrap()
}
