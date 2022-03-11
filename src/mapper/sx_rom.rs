use crate::cartridge::{Cartridge, BANK_SIZE};
use crate::types::*;

// TODO
pub struct SxRom {
  use_character_ram: bool,
  character_ram: Vec<Byte>,
  cart: Cartridge,
}
