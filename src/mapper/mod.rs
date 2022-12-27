pub mod cn_rom;
pub mod factory;
pub mod n_rom;
pub mod ux_rom;
pub mod sx_rom;

use serde::Serialize;

use crate::common::*;

pub trait Mapper {
  fn write_prg(&mut self, addr: Address, value: Byte);
  fn read_prg(&self, addr: Address) -> Byte;
  fn write_chr(&mut self, addr: Address, value: Byte);
  fn read_chr(&self, addr: Address) -> Byte;

  fn has_extended_ram(&self) -> bool;

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
