mod apu;
mod bus;
mod cartridge;
mod cpu;
pub mod instance;
mod mapper;
mod ppu;
mod controller;

pub type NesError = anyhow::Error;
pub type NesResult<T> = anyhow::Result<T, NesError>;

#[macro_use]
extern crate lazy_static;
extern crate serde;

pub mod plantform;
