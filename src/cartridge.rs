use crate::common::*;
use log::{error, info};
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::vec::Vec;

pub static BANK_SIZE: usize = 0x4000;

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
  prg_rom: Vec<Byte>,
  chr_rom: Vec<Byte>,
  name_table_mirroring: Byte,
  mapper_number: Byte,
  extended_ram: bool,
}

impl Cartridge {
  pub fn new() -> Self {
    Self {
      prg_rom: vec![],
      chr_rom: vec![],
      name_table_mirroring: 0,
      mapper_number: 0,
      extended_ram: false,
    }
  }

  fn read_from_header(&mut self, header: &[u8], banks: &mut Byte, vbanks: &mut Byte) -> bool {
    let header_str = std::str::from_utf8(header).expect("error when parse header to str");
    if !header_str.starts_with("NES\x1A") {
      error!(
        "Not a valid iNES image. Magic number: {:#x}{:#x}{:#x}{:#x} rather than NES1a",
        header[0], header[1], header[2], header[3]
      );
      return false;
    }
    *banks = header[4];
    if *banks == 0 {
      error!("ROM has no PRG-ROM banks. Loading ROM failed.");
      return false;
    }
    *vbanks = header[5];

    self.name_table_mirroring = header[6] & 0xB;
    self.mapper_number = ((header[6] >> 4) & 0xF) | (header[7] & 0xF0);
    self.extended_ram = bit_eq(header[6], 0x2);
    info!(
      "Load header finished. 16KB PRG-ROM Banks: {}, 8KB CHR-ROM Banks: {}",
      banks, vbanks
    );
    info!(
      "Name Table Mirroring: {}, Mapper: {}, Extended (CPU) RAM: {}",
      self.name_table_mirroring, self.mapper_number, self.extended_ram
    );
    if bit_eq(header[6], 0x4) {
      error!("Trainer is not supported.");
      return false;
    }

    if (header[0xA] & 0x3) != 0 {
      error!("PAL ROM not supported.");
      return false;
    } else {
      info!("ROM is NSTC compatible.");
    }

    return true;
  }

  pub fn load_from_data(&mut self, data: &[u8]) -> bool {
    let reader = BufReader::new(data);
    self.load_from_buf(reader)
  }

  pub fn load_from_file(&mut self, path_str: &str) -> bool {
    info!("Reading ROM content from {}", path_str);
    let path = Path::new(path_str);
    let rom_file_result = File::open(path);
    if let Err(_) = rom_file_result {
      error!("Can't the open ROM file {}", path_str);
      return false;
    }
    let file_reader = BufReader::new(rom_file_result.unwrap());
    self.load_from_buf(file_reader)
  }

  fn load_from_buf<T>(&mut self, mut reader: BufReader<T>) -> bool
  where
    T: std::io::Read,
  {
    let mut header = Vec::with_capacity(0x10);
    let mut banks = 0;
    let mut vbanks = 0;
    let mut read_size: usize;
    reader
      .by_ref()
      .take(0x10)
      .read_to_end(&mut header)
      .expect("Read ROM file failed");
    if !self.read_from_header(&mut header, &mut banks, &mut vbanks) {
      return false;
    }
    read_size = BANK_SIZE * banks as usize;
    self.prg_rom.reserve(read_size);
    if let Err(e) = reader
      .by_ref()
      .take(read_size as u64)
      .read_to_end(&mut self.prg_rom)
    {
      error!("Read ROM file failed {}", e);
      return false;
    }

    if vbanks != 0 {
      read_size = 0x2000 * vbanks as usize;
      self.chr_rom.reserve(read_size as usize);
      if let Err(e) = reader.take(read_size as u64).read_to_end(&mut self.chr_rom) {
        error!("Read ROM file failed {}", e);
        return false;
      }
    } else {
      info!("Cartridge with CHR-RAM");
    }
    true
  }

  pub fn get_rom(&self) -> &Vec<Byte> {
    return &self.prg_rom;
  }

  pub fn get_vrom(&self) -> &Vec<Byte> {
    &self.chr_rom
  }

  pub fn get_mapper(&self) -> Byte {
    return self.mapper_number;
  }

  pub fn get_name_table_mirroring(&self) -> Byte {
    return self.name_table_mirroring;
  }

  pub fn has_extended_ram(&self) -> bool {
    return self.extended_ram;
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn cartridge_test() {
    match crate::logger::init() {
      Err(_) => return,
      Ok(_) => {}
    };
    let mut cart = crate::cartridge::Cartridge::new();
    // assert!(cart.load_from_file("~/codebase/rust_NES/assets/test.nes"));
    assert!(cart.load_from_file("assets/test.nes"));
    println!("{}", cart.has_extended_ram());
  }
  fn modify_vec(vec: &mut Vec<u8>) {
    vec.push(1);
  }
  #[test]
  fn vec_test() {
    let mut cart = crate::cartridge::Cartridge::new();
    modify_vec(&mut cart.prg_rom);
    assert_eq!(cart.prg_rom.len(), 1);
  }
}
