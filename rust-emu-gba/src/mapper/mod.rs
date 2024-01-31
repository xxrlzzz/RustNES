use std::{cell::RefCell, rc::Rc};

use rust_emu_common::{component::cartridge::Cartridge, mapper::Mapper};

use crate::cartridge::GBACartridge;

enum MapperType {
  Rom(bool, bool),        // Rom(ram, battery)
  Mbc1(bool, bool),       // Mbc1(ram, battery)
  Mbc2(bool),             // Mbc2(battery)
  Mmm01(bool, bool),      // Mmm01(ram, battery)
  Mbc3(bool, bool, bool), // Mbc3(timer, ram, battery)
  Mbc5(bool, bool),       // Mbc5(ram, battery)
  Mbc5Rumble(bool, bool),
  Mbc6,
  Mbc7,
  PocketCamera,
  BandaiTama5,
  HuC3,
  HuC1,
}

pub(crate) mod mbc1;
pub(crate) mod rom_only;

pub fn create_mapper<'a>(cart: GBACartridge) -> Rc<RefCell<dyn Mapper + 'a>> {
  let mapper_type = cart.get_mapper();
  match mapper_type {
    0x00 => Rc::new(RefCell::new(rom_only::NRom::new(cart))),
    0x01 => Rc::new(RefCell::new(mbc1::MBC1::new(cart, false, false))),
    0x02 => Rc::new(RefCell::new(mbc1::MBC1::new(cart, true, false))),
    0x03 => Rc::new(RefCell::new(mbc1::MBC1::new(cart, true, true))),
    _ => panic!("invalid mapper type received {}", mapper_type),
  }
}
