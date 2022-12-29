mod apu;
mod bus;
mod cartridge;
mod common;
pub mod controller;
mod cpu;
pub mod emulator;
mod instance;
pub mod logger;
mod mapper;
mod ppu;
mod render;

pub type NesError = anyhow::Error;
pub type NesResult<T> = anyhow::Result<T, NesError>;

#[macro_use]
extern crate ini;
#[macro_use]
extern crate lazy_static;
extern crate serde;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(feature = "wasm")]
pub mod web;
