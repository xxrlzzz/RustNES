use crate::cartridge::Cartridge;
use crate::mapper::cn_rom::CnRom;
use crate::mapper::n_rom::NRom;
use crate::mapper::ux_rom::UxRom;
use crate::mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;

type MapperType = u8;
const NROM: MapperType = 0;
// static SXROM: MapperType = 1;
const UXROM: MapperType = 2;
const CNROM: MapperType = 3;

pub fn create_mapper<'a>(cartridge: Cartridge) -> Rc<RefCell<dyn Mapper + 'a>> {
  let mapper_type = cartridge.get_mapper();
  match mapper_type {
    NROM => Rc::new(RefCell::new(NRom::new(cartridge))),
    // SXROM => Box::new(SxRom::new(cartridge)),
    UXROM => Rc::new(RefCell::new(UxRom::new(cartridge))),
    CNROM => Rc::new(RefCell::new(CnRom::new(cartridge))),
    _ => {
      panic!("invalid mapper type received {}", mapper_type);
    }
  }
}
