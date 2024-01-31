mod apu;
mod bus;
mod cartridge;
// pub mod controller_b;
mod cpu;
// pub mod emulator_b;
pub mod instance;
mod mapper;
mod ppu;
// mod render_b;

pub type NesError = anyhow::Error;
pub type NesResult<T> = anyhow::Result<T, NesError>;

#[macro_use]
extern crate lazy_static;
extern crate serde;

pub mod plantform;
