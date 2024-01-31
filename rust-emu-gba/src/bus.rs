use std::{cell::RefCell, rc::Rc};

use rust_emu_common::{mapper::Mapper, types::*};
use serde::{Serialize, Deserialize};


#[derive(Default, Serialize, Deserialize)]
pub struct MainBus {
  #[serde(skip)]
  mapper: Option<Rc<RefCell<dyn Mapper>>>,

  #[serde(with = "serde_bytes")]
  wram: Vec<Byte>,
  #[serde(with = "serde_bytes")]
  hram: Vec<Byte>,
}

impl MainBus {

    pub fn new() -> Self {
        Self {
            wram: vec![0; 0x2000],
            hram: vec![0; 0x80],
            mapper: None
        }
    }

    pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
        self.mapper = Some(mapper);
    }
    /**
     * 
     * 0x0000 - 0x3FFF : ROM Bank 0
     * 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
     * 0x8000 - 0x97FF : CHR RAM
     * 0x9800 - 0x9BFF : BG Map 1
     * 0x9C00 - 0x9FFF : BG Map 2
     * 0xA000 - 0xBFFF : Cartridge RAM
     * 0xC000 - 0xCFFF : RAM Bank 0
     * 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
     * 0xE000 - 0xFDFF : Reserved - Echo RAM
     * 0xFE00 - 0xFE9F : Object Attribute Memory
     * 0xFEA0 - 0xFEFF : Reserved - Unusable
     * 0xFF00 - 0xFF7F : I/O Registers
     * 0xFF80 - 0xFFFE : Zero Page
     */
    pub fn read(&self, addr: Address) -> Byte {
        return match addr {
            // ROM data
            0x0000..=0x7FFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
            // Char/Map data
            0x8000..=0x9FFF => self.mapper.as_ref().unwrap().borrow().read_chr(addr),
            // Cartridge RAM
            0xA000..=0xBFFF => self.mapper.as_ref().unwrap().borrow().read_prg(addr),
            // WRAM (Working RAM)
            0xC000..=0xDFFF => self.wram_read(addr),
            // reserved echo ram...
            0xE000..=0xFDFF => 0,
            0xFE00..=0xFE9F => {
                //OAM
                // TODO
                0
            }
            0xFEA0..=0xFEFF => {
                //reserved unusable...
                0
            },
            //IO Registers...
            0xFF00..=0xFF7F => self.io_read(addr),
            //CPU enable register...
            // handle by cpu itself
            0xFFFF => 0,
            // no impl
            _ => self.hram_read(addr),
        }
    }

    pub fn io_read(&self, addr: Address) -> Byte {
        // TODO
        0
    }

    pub fn io_write(&mut self, addr: Address, value: Byte) {
        // TODO
    }

    pub fn write(&mut self, addr: Address, data: Byte) {
        match addr {
            // ROM data
            0x0000..=0x7FFF => self.mapper.as_ref().unwrap().borrow_mut().write_prg(addr, data),
            // Char/Map data
            0x8000..=0x9FFF => self.mapper.as_ref().unwrap().borrow_mut().write_chr(addr, data),
            // Cartridge RAM
            0xA000..=0xBFFF => self.mapper.as_ref().unwrap().borrow_mut().write_prg(addr, data),
            // WRAM (Working RAM)
            0xC000..=0xDFFF => self.wram_write(addr, data),
            // reserved echo ram...
            0xE000..=0xFDFF => (),
            0xFE00..=0xFE9F => {
                //OAM
                // TODO
                ()
            }
            0xFEA0..=0xFEFF => {
                //reserved unusable...
                ()
            },
            //IO Registers...
            0xFF00..=0xFF7F => self.io_write(addr, data),
            //CPU enable register...
            // handle by cpu itself
            0xFFFF => (),
            // no impl
            _ => self.hram_write(addr, data),
        }
    }

    fn wram_read(&self, addr: Address) -> Byte {
        if addr >= 0xE000 || addr < 0xC000 {
            log::error!("WRAM read out of range {}", addr);
            return 0;
        }
        self.wram[addr as usize - 0xC000]
    }

    fn wram_write(&mut self, addr: Address, value: Byte) {
        if addr >= 0xE000 || addr < 0xC000 {
            log::error!("WRAM write out of range {}", addr);
            return;
        }
        self.wram[addr as usize - 0xC000] = value;
    }

    fn hram_read(&self, addr: Address) -> Byte {
        if addr >= 0xFF80 || addr < 0xFF00 {
            log::error!("HRAM read out of range {}", addr);
            return 0;
        }
        self.hram[addr as usize - 0xFF80]
    }
    fn hram_write(&mut self, addr: Address, value: Byte) {
        if addr >= 0xFF80 || addr < 0xFF00 {
            log::error!("HRAM write out of range {}", addr);
            return;
        }
        self.hram[addr as usize - 0xFF80] = value;
    }
}
