use rust_emu_common::component::cartridge::Cartridge;
use rust_emu_common::types::*;
use log::{error, info};
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::vec::Vec;

pub static BANK_SIZE: usize = 1 << 10;

pub static RAM_SIZE_MAP: [usize; 6] = [
  0,
  0,
  8 * BANK_SIZE,
  32 * BANK_SIZE,
  128 * BANK_SIZE,
  64 * BANK_SIZE,
];

#[derive(Default)]
pub struct GBACartridge {
  prg_rom: Vec<Byte>,
  chr_rom: Vec<Byte>,
  title: [Byte; 16],
  new_lic_code: [Byte; 2],
  sgb_flag: Byte,
  cartridge_type: Byte,
  pub(crate) ram_size: usize,
}

impl GBACartridge {
  pub fn new() -> Self {
    Self {
      prg_rom: Vec::new(),
      chr_rom: Vec::with_capacity(0x2000),
      title: [0; 16],
      new_lic_code: [0; 2],
      sgb_flag: 0,
      cartridge_type: 0,
      ram_size: 0,
    }
  }

  fn read_header(&mut self, header: &[u8]) {
    self.title.copy_from_slice(&header[0x134..0x144]);
    self.new_lic_code = header[0x144..0x146]
      .try_into()
      .expect("error when parse header new lic code");
    self.sgb_flag = header[0x146];
    self.cartridge_type = header[0x147];
    let rom_size = header[0x148];
    self.prg_rom.reserve(32 * BANK_SIZE * (1 << rom_size));
    self.ram_size = RAM_SIZE_MAP[header[0x149] as usize];
    let destination = header[0x14A];
    let old_lic_code = header[0x14B];
    let mask_rom_version = header[0x14C];
    let header_checksum = header[0x14D];

    info!("GBA ROM INFO:");
    info!("title: {:?}", String::from_utf8_lossy(&self.title));
    info!("new lic code: {:?}", self.new_lic_code);
    info!("sgb flag: {:?}", self.sgb_flag);
    info!("cartridge type: {:?}", self.cartridge_type);
    info!("rom size: {:?}", rom_size);
    info!("destination: {:?}", destination);
    info!("old lic code: {:?}", old_lic_code);
    info!("mask rom version: {:?}", mask_rom_version);
    let mut calc_checksum :u8 = 0;
    for i in 0x134..0x14d {
      calc_checksum = calc_checksum.overflowing_sub(header[i] as u8).0;
      calc_checksum = calc_checksum.overflowing_sub(1).0;
    }
    if header_checksum == calc_checksum {
      info!("header checksum: OK");
    }
  }

  pub fn load_from_file(&mut self, path_str: &str) -> bool {
    let file = match File::open(path_str) {
      Err(why) => {
        error!("couldn't open file: {}", why);
        return false;
      }
      Ok(file) => file,
    };
    let mut file_reader = BufReader::new(file);
    let mut header = Vec::with_capacity(0x14e);
    file_reader.by_ref().take(0x14e).read_to_end(&mut header).expect("Read ROM file header failed");
    self.read_header(&header);
    let r = file_reader.rewind();
    if r.is_err() {
      error!("rewind failed");
      return false;
    }
    file_reader.by_ref().take(self.prg_rom.capacity() as u64).read_to_end(&mut self.prg_rom).expect("Read ROM file content failed");
    // self.chr_rom.extend(std::iter::repeat(0).take(0x2000));
    true
  }
}

impl Cartridge for GBACartridge {
    fn get_rom(&self) -> &Vec<Byte> {
        return &self.prg_rom;
    }

    fn get_vrom(&self) -> &Vec<Byte> {
        return &self.chr_rom;
    }

    fn get_mapper(&self) -> Byte {
      self.cartridge_type
    }

    fn get_name_table_mirroring(&self) -> Byte {
        0
    }

    fn has_extended_ram(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn cartridge_test() {
    match rust_emu_common::logger::init() {
      Err(_) => return,
      Ok(_) => {}
    };
    let mut cart = crate::cartridge::GBACartridge::new();
    assert!(cart.load_from_file("assets/Tetris(World).gb"));

  }
}
