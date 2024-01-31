use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};

use crate::cartridge::NESCartridge;
use crate::mapper::cn_rom::CnRom;
use crate::mapper::n_rom::NRom;
use crate::mapper::sx_rom::SxRom;
use crate::mapper::tx_rom::TxRom;
use crate::mapper::ux_rom::UxRom;
use std::{cell::RefCell, rc::Rc};

use super::{CNROM, NROM, SXROM, TXROM, UXROM};

use rust_emu_common::{component::cartridge::Cartridge, mapper::Mapper, types::*};

pub type MirrorCallback = Box<dyn FnMut(u8) -> ()>;
pub type IRQCallback = Box<dyn FnMut() -> ()>;

#[derive(
  Default, Debug, Clone, Copy, IntoPrimitive, FromPrimitive, PartialEq, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum NameTableMirroring {
  #[default]
  Horizontal = 0,
  Vertical = 1,
  FourScreen = 8,
  OneScreenLower,
  OneScreenHigher,
}

pub fn create_mapper<'a>(
  cartridge: NESCartridge,
  mirror_cb: MirrorCallback,
  irq_cb: IRQCallback,
) -> Rc<RefCell<dyn Mapper + 'a>> {
  let mapper_type = cartridge.get_mapper();
  match mapper_type {
    NROM => Rc::new(RefCell::new(NRom::new(cartridge))),
    SXROM => Rc::new(RefCell::new(SxRom::new(cartridge, mirror_cb))),
    UXROM => Rc::new(RefCell::new(UxRom::new(cartridge))),
    CNROM => Rc::new(RefCell::new(CnRom::new(cartridge))),
    TXROM => Rc::new(RefCell::new(TxRom::new(cartridge, mirror_cb, irq_cb))),
    _ => {
      panic!("invalid mapper type received {}", mapper_type);
    }
  }
}

pub fn load_mapper<'a>(
  mapper_type: Byte,
  serialized: &str,
  mirror_cb: MirrorCallback,
  irq_cb: IRQCallback,
) -> Rc<RefCell<dyn Mapper + 'a>> {
  match mapper_type {
    NROM => {
      let mapper_typed: NRom = serde_json::from_str(serialized).unwrap();
      Rc::new(RefCell::new(mapper_typed))
    }
    SXROM => {
      let mut mapper_typed: SxRom = serde_json::from_str(serialized).unwrap();
      mapper_typed.set_mirror_cb(mirror_cb);
      Rc::new(RefCell::new(mapper_typed))
    }
    UXROM => {
      let mapper_typed: UxRom = serde_json::from_str(serialized).unwrap();
      Rc::new(RefCell::new(mapper_typed))
    }
    CNROM => {
      let mapper_typed: CnRom = serde_json::from_str(serialized).unwrap();
      Rc::new(RefCell::new(mapper_typed))
    }
    // TODO TxRom
    TXROM => {
      let mut mapper_typed: TxRom = serde_json::from_str(serialized).unwrap();
      mapper_typed.set_mirror_cb(mirror_cb);
      mapper_typed.set_irq_cb(irq_cb);
      Rc::new(RefCell::new(mapper_typed))
    }
    _ => {
      panic!("invalid mapper type received {}", mapper_type);
    }
  }
}
