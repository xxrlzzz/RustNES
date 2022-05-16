use image::{Pixel, RgbaImage};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

mod palette_colors;

use crate::bus::main_bus::{
  IORegister, RegisterHandler, OAM_ADDR, OAM_DATA, PPU_ADDR, PPU_CTRL, PPU_DATA, PPU_MASK,
  PPU_SCROL, PPU_STATUS,
};
use crate::bus::message_bus::{Message, MessageBus};
use crate::bus::picture_bus::PictureBus;
use crate::common::serializer::*;
use crate::common::*;
use crate::mapper::Mapper;

#[derive(Copy, Clone, Serialize, Deserialize)]
enum PipelineState {
  PreRender,
  Render,
  PostRender,
  VerticalBlank,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum CharacterPage {
  Low,
  High,
}

pub const SCANLINE_END_CYCLE_LENGTH: u32 = 341;
const SCANLINE_END_CYCLE: usize = 340;
pub const VISIBLE_SCANLINES: usize = 240;
pub const SCANLINE_VISIBLE_DOTS: usize = 256;
const FRAME_END_SCANLINE: usize = 261;

// const ATTRIBUTE_OFFSET: u32 = 0x3C0;

// pixel processing unit
#[derive(Serialize, Deserialize)]
pub struct Ppu {
  bus: PictureBus,
  sprite_memory: Vec<Byte>,
  scanline_sprites: Vec<Byte>,

  pipeline_state: PipelineState,

  cycle: usize,
  scanline: usize,
  event_frame: bool,

  vblank: bool,
  sprite_zero_hit: bool,

  // Registers
  data_address: Address,
  temp_address: Address,
  fine_x_scroll: Byte,
  first_write: bool,
  data_buffer: Byte,

  sprite_data_address: usize,

  // Setup flags and variables
  long_sprites: bool,
  generate_interrupt: bool,

  grey_scale_mode: bool,
  show_sprites: bool,
  show_background: bool,
  hide_edge_sprites: bool,
  hide_edge_background: bool,

  background_page: CharacterPage,
  sprite_page: CharacterPage,

  data_address_increment: Address,
  #[serde(serialize_with = "rgba_ser", deserialize_with = "rgba_deser")]
  image: RgbaImage,
  #[serde(skip)]
  message_bus: Rc<RefCell<MessageBus>>,
}
impl Ppu {
  pub fn new(pic_bus: PictureBus, message_bus: Rc<RefCell<MessageBus>>) -> Self {
    Self {
      bus: pic_bus,
      sprite_memory: vec![0; 64 * 4],
      scanline_sprites: vec![],

      pipeline_state: PipelineState::PreRender,

      cycle: 0,
      scanline: 0,
      event_frame: false,
      vblank: false,
      sprite_zero_hit: false,

      data_address: 0,
      temp_address: 0,
      fine_x_scroll: 0,
      first_write: false,
      data_buffer: 0,

      sprite_data_address: 0,

      long_sprites: false,
      generate_interrupt: false,

      grey_scale_mode: false,
      show_sprites: false,
      show_background: false,
      hide_edge_sprites: false,
      hide_edge_background: false,

      background_page: CharacterPage::Low,
      sprite_page: CharacterPage::Low,

      data_address_increment: 0,

      image: RgbaImage::new(SCANLINE_VISIBLE_DOTS as u32, VISIBLE_SCANLINES as u32),
      message_bus,
    }
  }

  pub fn set_message_bus(&mut self, message_bus: Rc<RefCell<MessageBus>>) {
    self.message_bus = message_bus;
  }

  pub fn set_mapper_for_bus(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
    self.bus.set_mapper(mapper);
  }

  #[deprecated]
  pub fn reset(&mut self) {
    self.long_sprites = false;
    self.generate_interrupt = false;
    self.grey_scale_mode = false;
    self.vblank = false;

    self.show_background = true;
    self.show_sprites = true;
    self.event_frame = true;
    self.first_write = true;

    self.background_page = CharacterPage::Low;
    self.sprite_page = CharacterPage::Low;

    self.data_address = 0;
    self.cycle = 0;
    self.scanline = 0;
    self.sprite_data_address = 0;
    self.fine_x_scroll = 0;
    self.temp_address = 0;

    self.data_address_increment = 1;
    self.pipeline_state = PipelineState::PreRender;

    self.scanline_sprites.resize(0, 0);
  }

  pub fn step(&mut self) {
    match self.pipeline_state {
      PipelineState::PreRender => self.pre_render(),
      PipelineState::Render => self.render(),
      PipelineState::PostRender => self.post_render(),
      PipelineState::VerticalBlank => self.vertical_blank(),
    }
    self.cycle += 1;
  }

  fn pre_render(&mut self) {
    if self.cycle == 1 {
      self.vblank = false;
      self.sprite_zero_hit = false;
      return;
    }
    let enable_render = self.show_background && self.show_sprites;
    if enable_render {
      if self.cycle == SCANLINE_VISIBLE_DOTS + 2 {
        // Set bits related to horizontal position
        // Unset horizontal bits
        self.data_address &= !0x41F;
        // Copy
        self.data_address |= self.temp_address & 0x41F;
        return;
      } else if self.cycle > 280 && self.cycle <= 304 {
        // Set vertical bits
        // Unset bits related to horizontal
        self.data_address &= !0x7BE0;
        self.data_address |= self.temp_address & 0x7BE0;
        return;
      }
    }
    // If rendering is on, every other frame is one cycle shorter
    if self.cycle >= SCANLINE_END_CYCLE - (!self.event_frame && enable_render) as usize {
      self.pipeline_state = PipelineState::Render;
      self.cycle = 0;
      self.scanline = 0;
    }
  }

  fn render(&mut self) {
    if self.cycle > 0 && self.cycle <= SCANLINE_VISIBLE_DOTS {
      self.render_step1();
    } else if self.cycle == SCANLINE_VISIBLE_DOTS + 1 {
      if self.show_background {
        self.render_step2();
      }
    } else if self.cycle == SCANLINE_VISIBLE_DOTS + 2 {
      if self.show_background && self.show_sprites {
        // Copy bits related to horizontal position
        self.data_address &= !0x041F;
        self.data_address |= self.temp_address & 0x041F;
      }
    }
    if self.cycle >= SCANLINE_END_CYCLE {
      // Find and index sprites that are on the next Scanline
      // This isn't where/when this indexing, actually copying in 2C02 is done
      // but it shouldn't hurt any games if this is done here
      self.scanline_sprites.resize(0, 0);
      let range = if self.long_sprites { 16 } else { 8 };

      for i in self.sprite_data_address / 4..64 {
        let diff = self
          .scanline
          .overflowing_sub(self.sprite_memory[i * 4] as usize);
        if !diff.1 && diff.0 < range {
          self.scanline_sprites.push(i as Byte);
          if self.scanline_sprites.len() >= 8 {
            // 0..7
            break;
          }
        }
      }
      self.scanline += 1;
      self.cycle = 0;
    }

    if self.scanline >= VISIBLE_SCANLINES {
      self.pipeline_state = PipelineState::PostRender;
    }
  }

  fn render_step1(&mut self) {
    let mut bg_color = 0;
    let mut spr_color = 0;
    let mut bg_opaque = false;
    let mut spr_opaque = true;
    let mut sprite_foreground = false;

    let x = (self.cycle - 1) as u8;
    let y = self.scanline as i32;

    if self.show_background {
      let x_fine = (self.fine_x_scroll + x % 8) % 8;
      if !self.hide_edge_background || x >= 8 {
        // Fetch tile
        // Mask off fine y
        let mut addr = 0x2000 | (self.data_address & 0x0FFF);
        let tile = self.bus.read(addr) as Address;

        // Fetch pattern
        // Each pattern occupies 16 bytes, so multiply by 16
        // Add fine y
        addr = (tile * 16) + ((self.data_address >> 12) & 0x7);
        // Set whether the pattern is in the high or low page
        addr |= (self.background_page as u16) << 12;
        // Get the corresponding bit determined by (8 - x_fine)
        // from the right bit 0 of palette entry
        bg_color = (self.bus.read(addr) >> (7 ^ x_fine)) & 1;
        // bit 1
        bg_color |= (self.bus.read(addr + 8) >> (7 ^ x_fine) & 1) << 1;

        // flag used to calculate final pixel with the sprite pixel
        bg_opaque = bg_color != 0;

        // fetch attribute and calculate higher two bits of palette
        addr = 0x23C0
          | (self.data_address & 0x0C00)
          | ((self.data_address >> 4) & 0x38)
          | ((self.data_address >> 2) & 0x07);
        let attribute = self.bus.read(addr);
        let shift = (self.data_address >> 4) & 4 | (self.data_address & 2);
        // Extract and set the upper two bits for the color
        bg_color |= ((attribute >> shift) & 0x3) << 2;
      }
      // Increment/wrap coarse X
      if x_fine == 7 {
        // if coarse X = 31
        if self.data_address & 0x1F == 31 {
          // coarse X = 0
          self.data_address &= !0x1F;
          // switch horizontal name table
          self.data_address ^= 0x0400;
        } else {
          self.data_address += 1;
        }
      }
    }

    if self.show_sprites && (!self.hide_edge_sprites || x >= 8) {
      for i in &self.scanline_sprites {
        let idx = (i * 4) as usize;
        let spr_x = self.sprite_memory[idx + 3];
        let diff_x = x.overflowing_sub(spr_x);
        if diff_x.1 || diff_x.0 >= 8 {
          continue;
        }

        let spr_y = self.sprite_memory[idx] + 1;
        let tile = self.sprite_memory[idx + 1] as u16;
        let attribute = self.sprite_memory[idx + 2];

        let length = if self.long_sprites { 16 } else { 8 };
        let mut x_shift = diff_x.0 as Byte;
        // NOTE : what if y small than spr_y?
        let mut y_offset = (y - spr_y as i32) % length;

        // if not flipping horizontally
        if !bit_eq(attribute, 0x40) {
          x_shift ^= 7;
        }
        // if flipping vertically
        if bit_eq(attribute, 0x80) {
          y_offset ^= length - 1;
        }
        let mut addr: Address;

        if !self.long_sprites {
          addr = tile * 16 + y_offset as u16;
          if let CharacterPage::High = self.sprite_page {
            addr += 0x1000;
          }
        } else {
          // 8 * 16 sprites
          // bit-3 is one if it is the bottom tile of the sprite
          // multiply by two to get the next pattern
          y_offset = (y_offset & 7) | ((y_offset & 8) << 1);
          addr = ((tile >> 1) * 32 + y_offset as u16) as Address;
          // Bank 0x1000 if bit-0 is high
          addr |= ((tile & 1) as Address) << 12;
        }

        // bit 0 of palette entry
        spr_color |= (self.bus.read(addr) >> x_shift) & 1;
        // bit 1
        spr_color |= ((self.bus.read(addr + 8) >> x_shift) & 1) << 1;

        spr_opaque = spr_color != 0;
        if !spr_opaque {
          // assert_eq!(spr_color, 0);
          continue;
        }

        // Select sprite palette
        spr_color |= 0x10;
        // bits 2-3
        spr_color |= (attribute & 0x3) << 2;

        sprite_foreground = !bit_eq(attribute, 0x20);

        // Sprite-0 hit detection
        self.sprite_zero_hit |= self.show_background && *i == 0 && bg_opaque;
        break;
      }
    }
    let palette_addr = if spr_opaque && (!bg_opaque || sprite_foreground) {
      spr_color
    } else if bg_opaque {
      bg_color
    } else {
      0
    };
    let color = palette_colors::COLORS[self.bus.read_palette(palette_addr) as usize];
    self.image.put_pixel(x as u32, y as u32, color.to_rgba())
  }

  fn render_step2(&mut self) {
    // If fine Y < 7
    if !bit_eq(self.data_address, 0x7000) {
      // Increment fine Y
      self.data_address += 0x1000;
    } else {
      // Fine Y = 0
      self.data_address &= !0x7000;
      // let y = coarse y
      let mut y = (self.data_address & 0x03E0) >> 5;
      if y == 29 {
        // coarse y = 0;
        y = 0;
        // switch vertical name table
        self.data_address ^= 0x0800;
      } else if y == 31 {
        y = 0;
      } else {
        y += 1;
      }
      // put coarse y back into data_address
      self.data_address = (self.data_address & !0x03E0) | (y << 5);
    }
  }

  fn post_render(&mut self) {
    if self.cycle < SCANLINE_END_CYCLE {
      return;
    }
    self.scanline += 1;
    self.cycle = 0;
    self.pipeline_state = PipelineState::VerticalBlank;
    self
      .message_bus
      .borrow_mut()
      .push(Message::PpuRender(self.image.clone()));
    // log::info!("post render");
  }

  fn vertical_blank(&mut self) {
    if self.cycle == 1 && self.scanline == (VISIBLE_SCANLINES + 1) {
      self.vblank = true;
      if self.generate_interrupt {
        self.message_bus.borrow_mut().push(Message::CpuInterrupt);
      }
    }

    if self.cycle >= SCANLINE_END_CYCLE {
      self.scanline += 1;
      self.cycle = 0;
    }

    if self.scanline >= FRAME_END_SCANLINE {
      self.pipeline_state = PipelineState::PreRender;
      self.scanline = 0;
      self.event_frame = !self.event_frame;
    }
  }

  pub fn get_status(&mut self) -> Byte {
    let status = ((self.sprite_zero_hit as Byte) << 6) | ((self.vblank as Byte) << 7);
    self.vblank = false;
    self.first_write = true;
    return status;
  }

  pub fn get_data(&mut self) -> Byte {
    let mut data = self.bus.read(self.data_address);
    self.data_address += self.data_address_increment;
    // Reads are delayed by one byte/read when address is in the range
    if self.data_address < 0x3F00 {
      std::mem::swap(&mut self.data_buffer, &mut data);
    }
    data
  }

  // 0x2004: OAMDATA (read)
  pub fn get_oam_data(&self) -> Byte {
    self.sprite_memory[self.sprite_data_address]
  }

  pub fn set_data_address(&mut self, addr: Address) {
    if self.first_write {
      // Unset the upper byte
      self.temp_address = (self.temp_address & 0x80FF) | ((addr & 0x3F) << 8);
    } else {
      // Unset the lower byte
      self.temp_address = (self.temp_address & 0xFF00) | addr;
      self.data_address = self.temp_address;
    }
    self.first_write = !self.first_write;
  }

  pub fn set_oam_address(&mut self, addr: Byte) {
    self.sprite_data_address = addr as usize;
  }

  //  0x2004: OAMDATA (write)
  pub fn set_oam_data(&mut self, value: Byte) {
    self.sprite_memory[self.sprite_data_address] = value;
    self.sprite_data_address += 1;
  }

  pub fn set_data(&mut self, value: Byte) {
    self.bus.write(self.data_address, value);
    self.data_address += self.data_address_increment;
  }

  pub fn set_scroll(&mut self, scroll: Byte) {
    let scroll = scroll as Address;
    if self.first_write {
      self.temp_address &= !0x001F;
      self.temp_address |= (scroll >> 3) & 0x001F;
      self.fine_x_scroll = scroll as Byte & 0x7;
    } else {
      self.temp_address &= !0x73e0;
      self.temp_address |= (scroll & 0x7) << 12 | ((scroll & 0xF8) << 2);
    }
    self.first_write = !self.first_write;
  }

  // 0x2001 PPUMASK
  pub fn set_mask(&mut self, mask: Byte) {
    self.grey_scale_mode = bit_eq(mask, 0x1);
    self.hide_edge_background = !bit_eq(mask, 0x2);
    self.hide_edge_sprites = !bit_eq(mask, 0x4);
    self.show_background = bit_eq(mask, 0x8);
    self.show_sprites = bit_eq(mask, 0x10);
  }

  /**
   0x2000 PPUCTRL
  7  bit  0
  ---- ----
  VPHB SINN
  |||| ||||
  |||| ||++- Base nametable address
  |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
  |||| |+--- VRAM address increment per CPU read/write of PPUDATA
  |||| |     (0: add 1, going across; 1: add 32, going down)
  |||| +---- Sprite pattern table address for 8x8 sprites
  ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
  |||+------ Background pattern table address (0: $0000; 1: $1000)
  ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
  |+-------- PPU master/slave select
  |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
  +--------- Generate an NMI at the start of the
            vertical blanking interval (0: off; 1: on)
   */
  pub fn control(&mut self, ctrl: Byte) {
    self.generate_interrupt = bit_eq(ctrl, 0x80);
    // Ignore the 6 bit.
    self.long_sprites = bit_eq(ctrl, 0x20);
    self.background_page = if bit_eq(ctrl, 0x10) {
      CharacterPage::High
    } else {
      CharacterPage::Low
    };
    self.sprite_page = if bit_eq(ctrl, 0x8) {
      CharacterPage::High
    } else {
      CharacterPage::Low
    };
    self.data_address_increment = if bit_eq(ctrl, 0x4) { 0x20 } else { 1 };
    // Set the name table in the temp address, this will be reflected
    // in the data address during rendering
    self.temp_address = (self.temp_address & !0xC00) | ((ctrl as Address & 0x3) << 10);
  }

  pub unsafe fn do_dma(&mut self, page: *const Byte) {
    for i in self.sprite_data_address..SCANLINE_VISIBLE_DOTS {
      self.sprite_memory[i] = *page.add(i);
    }
    for i in 0..self.sprite_data_address {
      self.sprite_memory[i] = *page.add(i + SCANLINE_VISIBLE_DOTS - self.sprite_data_address);
    }
  }
}

impl RegisterHandler for Ppu {
  fn read(&mut self, address: IORegister) -> Option<Byte> {
    match address {
      PPU_STATUS => Some(self.get_status()),
      PPU_DATA => Some(self.get_data()),
      OAM_ADDR => Some(self.get_oam_data()),
      _ => None,
    }
  }

  fn write(&mut self, address: IORegister, value: Byte) -> bool {
    match address {
      PPU_CTRL => self.control(value),
      PPU_MASK => self.set_mask(value),
      OAM_ADDR => self.set_oam_address(value),
      OAM_DATA => self.set_oam_data(value),
      PPU_SCROL => self.set_scroll(value),
      PPU_ADDR => self.set_data_address(value as Address),
      PPU_DATA => self.set_data(value),
      _ => return false,
    }
    true
  }

  fn dma(&mut self, page: *const Byte) -> bool {
    unsafe {
      self.do_dma(page as *const Byte);
    }
    true
  }
}
