use crate::types::*;
use std::vec::Vec;

pub trait Cartridge {
    fn get_rom(&self) -> &Vec<Byte>;
    // video rom
    fn get_vrom(&self) -> &Vec<Byte>;
    fn get_mapper(&self) -> Byte;
    fn get_name_table_mirroring(&self) -> Byte;
    fn has_extended_ram(&self) -> bool;
  }
  