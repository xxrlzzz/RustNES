pub mod cn_rom;
pub mod factory;
pub mod n_rom;
pub mod ux_rom;
pub mod sx_rom;
pub mod tx_rom;


type MapperType = u8;
pub(crate) const NROM: MapperType = 0;
pub(crate) const SXROM: MapperType = 1;
pub(crate) const UXROM: MapperType = 2;
pub(crate) const CNROM: MapperType = 3;
pub(crate) const TXROM: MapperType = 4;
// TODO
// pub(crate) const EXROM: MapperType = 5;
// pub(crate) const AXROM: MapperType = 7;
// pub(crate) const PXROM: MapperType = 9;
