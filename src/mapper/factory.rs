use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};

use crate::cartridge::Cartridge;
use crate::common::Byte;
use crate::mapper::cn_rom::CnRom;
use crate::mapper::n_rom::NRom;
use crate::mapper::ux_rom::UxRom;
use crate::mapper::Mapper;
use std::sync::{Arc, Mutex};

use super::sx_rom::SxRom;

type MapperType = u8;
pub(crate) const NROM: MapperType = 0;
pub(crate) const SXROM: MapperType = 1;
pub(crate) const UXROM: MapperType = 2;
pub(crate) const CNROM: MapperType = 3;

pub type MirrorCallback = Box<dyn FnMut(u8) -> () + Sync + Send>;

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
  cartridge: Cartridge,
  mirror_cb: MirrorCallback,
) -> Arc<Mutex<dyn Mapper + 'a + Sync + Send>> {
  let mapper_type = cartridge.get_mapper();
  match mapper_type {
    NROM => Arc::new(Mutex::new(NRom::new(cartridge))),
    SXROM => Arc::new(Mutex::new(SxRom::new(cartridge, mirror_cb))),
    UXROM => Arc::new(Mutex::new(UxRom::new(cartridge))),
    CNROM => Arc::new(Mutex::new(CnRom::new(cartridge))),
    _ => {
      panic!("invalid mapper type received {}", mapper_type);
    }
  }
}

pub fn load_mapper<'a>(
  mapper_type: Byte,
  serialized: &str,
  mirror_cb: MirrorCallback,
) -> Arc<Mutex<dyn Mapper + 'a + Send + Sync>> {
  match mapper_type {
    NROM => {
      let mapper_typed: NRom = serde_json::from_str(serialized).unwrap();
      Arc::new(Mutex::new(mapper_typed))
    }
    SXROM => {
      let mut mapper_typed: SxRom = serde_json::from_str(serialized).unwrap();
      mapper_typed.set_mirror_cb(mirror_cb);
      Arc::new(Mutex::new(mapper_typed))
    }
    UXROM => {
      let mapper_typed: UxRom = serde_json::from_str(serialized).unwrap();
      Arc::new(Mutex::new(mapper_typed))
    }
    CNROM => {
      let mapper_typed: CnRom = serde_json::from_str(serialized).unwrap();
      Arc::new(Mutex::new(mapper_typed))
    }
    _ => {
      panic!("invalid mapper type received {}", mapper_type);
    }
  }
}
