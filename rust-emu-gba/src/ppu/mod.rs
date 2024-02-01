use std::sync::mpsc;

use image::{Rgba, RgbaImage, GenericImage};
use rust_emu_common::types::*;
use serde::{Deserialize, Serialize};
use crate::{instance::Message, interrupt, picture_bus::PictureBus};

use self::{
  lcd::{Lcd, LcdMode, StatSrc},
  pipeline::{FetchState, PixelFiFo},
};

mod lcd;
mod pipeline;

static LINES_PER_FRAME: u8 = 154;
static TICKS_PER_LINE: u16 = 456;
static YRES: u8 = 144;
static XRES: u8 = 160;

pub fn is_between<T: PartialOrd + Copy + std::ops::Add<Output = T>>(a: T, b: T, c: T) -> bool {
  b <= a && a < b + c
}

#[derive(Default, Clone, Copy)]
pub(crate) struct OamEntry {
  pub(crate) y: Byte,
  pub(crate) x: Byte,
  pub(crate) title: Byte,
  /*
  Bit7   BG and Window over OBJ (0=No, 1=BG and Window colors 1-3 over the OBJ)
  Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
  Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
  Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
  Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
  Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
  */
  pub(crate) flag: Byte,
}

impl OamEntry {
  pub(crate) fn priority(&self) -> bool {
    bit_eq(self.flag, 7)
  }
  pub(crate) fn y_flip(&self) -> bool {
    bit_eq(self.flag, 6)
  }

  pub(crate) fn x_flip(&self) -> bool {
    bit_eq(self.flag, 5)
  }

  pub(crate) fn dmg(&self) -> bool {
    bit_eq(self.flag, 4)
  }
  // pub(crate) fn bank(&self) -> bool {
  //   bit_eq(self.flag, 3)
  // }
  // pub(crate) fn cgb(&self) -> Byte {
  //   self.flag & 0x3
  // }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GBAPpu {
  #[serde(skip)]
  picture_bus: PictureBus,
  #[serde(skip)]
  pfc: PixelFiFo,

  line_sprite_count: Byte,

  window_line: Byte,

  current_frame: u32,
  line_ticks: u16,
  #[serde(skip)]
  lcd: Lcd,

  #[serde(skip)]
  pub(crate) image: RgbaImage, // not save image for now
  #[serde(skip)]
  message_sx: Option<mpsc::Sender<Message>>, // interrupt channel
}

impl GBAPpu {
  pub fn new(message_sx: mpsc::Sender<Message>) -> Self {
    Self {
      picture_bus: PictureBus::new(),
      pfc: PixelFiFo::new(),
      line_sprite_count: 0,
      window_line: 0,
      current_frame: 0,
      line_ticks: 0,

      lcd: Lcd::new(),

      image: RgbaImage::new(XRES as u32, YRES as u32),
      message_sx: Some(message_sx),
    }
  }

  pub fn step(&mut self) {
    self.line_ticks += 1;
    // log::info!("ppu step {} {}", self.lcd.mode() as Byte, self.line_ticks);
    match self.lcd.mode() {
      lcd::LcdMode::hblank => self.handle_hblank(),
      lcd::LcdMode::vblank => self.handle_vblank(),
      lcd::LcdMode::oam => self.handle_oma(),
      lcd::LcdMode::transfer => self.handle_xfer(),
    }
  }

  pub fn handle_oma(&mut self) {
    if self.line_ticks >= 80 {
      self.lcd.set_mode(LcdMode::transfer);
      self.pfc.cur_fetch_state = FetchState::Tile;
      self.pfc.line_x = 0;
      self.pfc.fetch_x = 0;
      self.pfc.pushed_x = 0;
      self.pfc.fifo_x = 0;
    }

    if self.line_ticks == 1 {
      //read oam on the first tick only...
      // self.line_sprites = None;
      self.line_sprite_count = 0;

      self.load_line_sprites();
    }
  }

  pub fn handle_xfer(&mut self) {
    self.pipeline_process();

    if self.pfc.pushed_x >= XRES {
      self.pfc.fifo_reset();

      self.lcd.set_mode(LcdMode::hblank);
      if self.lcd.stat_int(StatSrc::hblank) {
        self.send_message(Message::CpuInterrupt(interrupt::INT_LCD));
      }
    }
  }

  pub fn handle_vblank(&mut self) {
    if self.line_ticks < TICKS_PER_LINE {
      return;
    }
    self.increment_ly();

    if self.lcd.ly >= LINES_PER_FRAME {
      self.lcd.set_mode(LcdMode::oam);
      self.lcd.ly = 0;
      self.window_line = 0;
    }

    self.line_ticks = 0;
  }

  pub fn handle_hblank(&mut self) {
    if self.line_ticks < TICKS_PER_LINE {
      return;
    }
    self.increment_ly();
    if self.lcd.ly >= YRES {
      self.lcd.set_mode(LcdMode::vblank);
      self.send_message(Message::CpuInterrupt(interrupt::INT_VBLANK));

      if self.lcd.stat_int(StatSrc::vblank) {
        self.send_message(Message::CpuInterrupt(interrupt::INT_LCD));
      }

      self.current_frame += 1;

      //calc FPS
      //
    } else {
      self.lcd.set_mode(LcdMode::oam);
    }
    self.line_ticks = 0;
  }

  fn load_line_sprites(&mut self) {
    let cur_y = self.lcd.ly;

    let sprite_height = self.lcd.obj_height();

    for i in 0..40 {
      let entry = &self.picture_bus.oam_ram()[i];
      if entry.x == 0 {
        //x = 0 means not visible...
        continue;
      }
      if self.line_sprite_count >= 10 {
        //max 10 sprites per line...
        break;
      }
      if entry.y <= cur_y + 16 && entry.y + sprite_height > cur_y {
        //this sprite is on the current line.
        // TODO handle ome line entry
      }
    }
  }

  fn increment_ly(&mut self) {
    if self.lcd.window_visible()
      && self.lcd.ly >= self.lcd.win_y
      && self.lcd.ly <= self.lcd.win_y + YRES
    {
      self.window_line += 1;
    }

    self.lcd.ly += 1;

    if self.lcd.ly == self.lcd.ly_compare {
      self.lcd.set_lyc(true);

      if self.lcd.stat_int(StatSrc::lyc) {
        self.message_sx.as_ref().unwrap().send(Message::CpuInterrupt(interrupt::INT_LCD)).unwrap();
      }
    } else {
      self.lcd.set_lyc(false);
    }
  }

  fn pipeline_process(&mut self) {
    self.pfc.map_y = self.lcd.ly + self.lcd.scroll_y;
    self.pfc.map_x = self.pfc.fetch_x + self.lcd.scroll_x;
    self.pfc.tile_y = (self.pfc.map_y % 8) * 2;

    if !bit_eq(self.line_ticks, 1) {
      self.pfc.fetch(&mut self.lcd, &self.picture_bus, self.window_line);
    }
    if let Some((offset, pixel)) =  self.pfc.push_pixel(&self.lcd) {
      // 
      unsafe {
        let color = Rgba([(pixel & 0xC0 >> 6) as u8, (pixel & 0x30 >> 4) as u8, (pixel & 0x0C >> 2) as u8, (pixel & 0x03) as u8]);
        self.image.unsafe_put_pixel(offset as u32, self.lcd.ly as u32, color);
      }
    }
  }

  fn send_message(&mut self, msg : Message) {
    self.message_sx.as_ref().unwrap().send(msg).unwrap();
  }
}
