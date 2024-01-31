use rust_emu_common::component::cartridge::Cartridge;
use rust_emu_common::mapper::Mapper;
use rust_emu_common::types::*;

use crate::cartridge::GBACartridge;

pub struct MBC1 {
  cart: GBACartridge,
  rom_bank_enable: bool,
  ram_banking: bool,
  rom_bank_value: u8,
  ram_bank_value: u8,
  rom_bank_x: Address,
  banking_mode: bool,

  ram_bank: Option<Byte>,
  ram_banks: Vec<[Byte; 0x2000]>,

  need_save: bool,

  character_ram: [Byte; 0x2000],

  // props
  ram_enable: bool,
  battery_enable: bool,
}

impl MBC1 {
  pub fn new(cart: GBACartridge, ram_enable: bool, battery_enable: bool) -> MBC1 {
    let bank_size = cart.ram_size;
    let mut ram_banks = vec![];
    for _ in 0..8 {
      ram_banks.push([0; 0x2000]);
    }
    MBC1 {
      cart,
      ram_banking: false,
      rom_bank_enable: false,
      rom_bank_value: 0,
      ram_bank_value: 0,
      rom_bank_x: 0x4000,
      banking_mode: false,
      
      ram_bank: None,
      ram_banks,

      need_save: false,
      
      character_ram: [0; 0x2000],
      
      ram_enable,
      battery_enable,
    }
  }

  fn battery_save(&self) {
    if self.ram_bank.is_none() {
      return;
    }
    if !self.need_save {
      return;
    }
    // TODO impl save ram_bank
  }
}

impl Mapper for MBC1 {
  fn write_prg(
    &mut self,
    addr: Address,
    mut data: Byte,
  ) {
    if addr < 0x2000 {
      self.ram_enable = (data & 0x0F) == 0x0A;
    }

    if (addr & 0xE000) == 0x2000 {
      //rom bank number
      if data == 0 {
        data = 1;
      }

      data &= 0b11111;

      self.rom_bank_value = data;
      self.rom_bank_x = 0x4000 * self.rom_bank_value as Address;
    }

    if (addr & 0xE000) == 0x4000 {
      //rom bank number
      self.ram_bank_value = data & 0x3;

      if self.ram_banking {
        self.battery_save();
        
        self.ram_bank = Some(self.ram_bank_value);
      }
    }

    if (addr & 0xE000) == 0x6000 {
      //banking mode select
      self.banking_mode = bit_eq(data, 1);

      self.ram_banking = self.banking_mode;

      if self.ram_banking {
        self.battery_save();
        
        self.ram_bank = Some(self.ram_bank_value);
      }
    }

    if (addr & 0xE000) == 0xA000 {
      if !self.ram_enable {
        return;
      }
      
      if let Some(rb_value) = self.ram_bank {
        self.ram_banks[rb_value as usize][addr as usize - 0xA000] = data;
  
        if self.battery_enable {
          self.need_save = true;
        }
      }
    }
  }

  fn read_prg(&self, addr: Address) -> Byte {
    if addr < 0x4000 {
      return self.cart.get_rom()[addr as usize];
    }

    if (addr & 0xE000) == 0xA000 {
      if !self.ram_enable {
        return 0xFF;
      }

      if let Some(rb) = self.ram_bank {
        return self.ram_banks[rb as usize][addr as usize - 0xA000];
      } else {
        return 0xFF;
      }
    }

    self.cart.get_rom()[(self.rom_bank_x + addr) as usize - 0x4000]
  }

  fn write_chr(
    &mut self,
    addr: Address,
    value: Byte,
  ) {
    self.character_ram[addr as usize - 0x8000] = value;
  }

  fn read_chr(&self, addr: Address) -> Byte {
    self.character_ram[addr as usize - 0x8000]
  }

  fn has_extended_ram(&self) -> bool {
    todo!()
  }

  fn get_name_table_mirroring(&self) -> u8 {
    todo!()
  }

  fn save(&self) -> String {
    todo!()
  }

  fn mapper_type(&self) -> u8 {
    todo!()
  }
}
