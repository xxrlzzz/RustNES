use rust_emu_common::types::*;

use super::YRES;

#[repr(u8)]
pub(crate) enum LcdMode {
  hblank,
  vblank,
  oam,
  transfer,
}

impl From<u8> for LcdMode {
  fn from(value: u8) -> Self {
    match value {
      0 => LcdMode::hblank,
      1 => LcdMode::vblank,
      2 => LcdMode::oam,
      3 => LcdMode::transfer,
      _ => panic!("Invalid LCD mode: {}", value),
    }
  }
}

impl Into<u8> for LcdMode {
  fn into(self) -> u8 {
    match self {
      LcdMode::hblank => 0,
      LcdMode::vblank => 1,
      LcdMode::oam => 2,
      LcdMode::transfer => 3,
    }
  }
}

#[repr(u8)]
pub(crate) enum StatSrc {
    hblank,
    vblank,
    oam,
    lyc,
}

impl From<u8> for StatSrc {
  fn from(value: u8) -> Self {
    match value {
      0x0F=> StatSrc::hblank,
      0x10 => StatSrc::vblank,
      0x20=> StatSrc::oam,
      0x40 => StatSrc::lyc,
      _ => panic!("Invalid LCD mode: {}", value),
    }
  }
}

impl Into<u8> for StatSrc {
  fn into(self) -> u8 {
    match self {
      StatSrc::hblank => 0xF,
      StatSrc::vblank => 0x10,
      StatSrc::oam => 0x20,
      StatSrc::lyc => 0x40,
    }
  }
}


#[derive(Default)]
pub(crate) struct Lcd {
  // registers
  lcdc: Byte,
  lcds: Byte,
  pub scroll_y: Byte,
  pub scroll_x: Byte,
  pub ly: Byte,
  pub ly_compare: Byte,
  dma: Byte,
  bg_palette: Byte,
  obj_palette: [Byte; 2],
  pub win_y: Byte,
  pub win_x: Byte,

  // other data...
  pub bg_colors: [u32; 4],
  pub sp1_colors: [u32; 4],
  pub sp2_colors: [u32; 4],
}

static COLORS_DEFAULT: [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];

impl Lcd {
  pub fn new() -> Self {
    Self {
      lcdc: 0x91,
      lcds: 0,
      scroll_y: 0,
      scroll_x: 0,
      ly: 0,
      ly_compare: 0,
      dma: 0,
      bg_palette: 0xFC,
      obj_palette: [0xFF; 2],
      win_y: 0,
      win_x: 0,
      bg_colors: COLORS_DEFAULT,
      sp1_colors: COLORS_DEFAULT,
      sp2_colors: COLORS_DEFAULT,
    }
  }

  pub fn read(&self, addr: Address) -> Byte {
    match addr {
      0x0 => self.lcdc,
      0x1 => self.lcds,
      0x2 => self.scroll_y,
      0x3 => self.scroll_x,
      0x4 => self.ly_compare,
      0x5 => self.dma,
      0x6 => self.bg_palette,
      0x7 => self.obj_palette[0],
      0x8 => self.obj_palette[1],
      0x9 => self.win_x,
      0xA => self.win_y,
      _ => 0,
    }
  }

  fn update_palette(&mut self, palette: Byte, pal_switch: Byte) {
    let colors = match pal_switch {
      1 => &mut self.sp1_colors,
      2 => &mut self.sp2_colors,
      _ => &mut self.bg_colors,
    };
    colors[0] = COLORS_DEFAULT[(palette & 0x3) as usize];
    colors[0] = COLORS_DEFAULT[((palette >> 2) & 0x3) as usize];
    colors[0] = COLORS_DEFAULT[((palette >> 4) & 0x3) as usize];
    colors[0] = COLORS_DEFAULT[((palette >> 6) & 0x3) as usize];
  }

  pub fn write(&mut self, addr: Address, data: Byte) {
    match addr - 0xFF40 {
      0x0 => {
        self.lcdc = data;
      }
      0x1 => {
        self.lcds = data;
      }
      0x2 => {
        self.scroll_y = data;
      }
      0x3 => {
        self.scroll_x = data;
      }
      0x4 => {
        self.ly_compare = data;
      }
      0x5 => {
        self.dma = data;
      }
      0x6 => {
        self.bg_palette = data;
        self.dma_start(data);
      }
      0x7 => {
        self.obj_palette[0] = data;
        self.update_palette(data & 0xFC, 0);
      }
      0x8 => {
        self.obj_palette[1] = data;
        self.update_palette(data & 0xFC, 1);
      }
      0x9 => {
        self.win_x = data;
        self.update_palette(data & 0xFC, 2);
      }
      0xA => {
        self.win_y = data;
      }
      _ => {}
    }
  }

  fn dma_start(&mut self, data: Byte) {
    // TODO
  }

  pub(crate) fn bgw_enable(&self) -> bool {
    bit_eq(self.lcdc, 1 << 0)
  }

  pub(crate) fn obj_enable(&self) -> bool {
    bit_eq(self.lcdc, 1 << 1)
  }

  pub(crate) fn obj_height(&self) -> Byte {
    if bit_eq(self.lcdc, 1 << 2) {
      16
    } else {
      8
    }
  }

  pub(crate) fn bg_map_area(&self) -> Address {
    if bit_eq(self.lcdc, 1 << 3) {
      0x9C00
    } else {
      0x9800
    }
  }

  pub(crate) fn bgw_data_area(&self) -> Address {
    if bit_eq(self.lcdc, 1 << 4) {
      0x8000
    } else {
      0x8800
    }
  }

  pub(crate) fn win_enable(&self) -> bool {
    bit_eq(self.lcdc, 5)
  }

  pub(crate) fn win_map_area(&self) -> Address {
    if bit_eq(self.lcdc, 6) {
      0x9C00
    } else {
      0x9800
    }
  }

  pub(crate) fn lcd_enable(&self) -> bool {
    bit_eq(self.lcdc, 7)
  }

  pub(crate) fn mode(&self) -> LcdMode {
    (self.lcds & 0x3).into()
  }

  pub(crate) fn set_mode(&mut self, mode: LcdMode) {
    self.lcds = (self.lcdc & 0xFC) | (mode as Byte & 0x3)
  }

  pub(crate) fn lyc(&self) -> bool {
    bit_eq(self.lcds, 1 << 2)
  }

  pub(crate) fn set_lyc(&mut self, lyc: bool) {
    self.lcds = (self.lcds & 0xFD) | ((lyc as Byte) >> 2)
  }

  pub(crate) fn stat_int(&self, stat: StatSrc) -> bool {
    self.lcds & (stat as u8) != 0
  }

  pub(crate) fn window_visible(&self) -> bool {
    self.win_enable() && self.win_x >= 0 && self.win_x < 167 && self.win_y >= 0 && self.win_y < YRES
  }
}
