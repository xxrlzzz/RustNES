use std::collections::VecDeque;

use rust_emu_common::types::*;

use crate::picture_bus::PictureBus;

use super::{is_between, lcd::Lcd, OamEntry, XRES, YRES};

#[derive(Default)]
pub(crate) enum FetchState {
  #[default]
  Tile,
  Data0,
  Data1,
  Idle,
  Push,
}

#[derive(Default)]
pub(crate) struct PixelFiFo {
  pub(crate) cur_fetch_state: FetchState,
  pixel_fifo: VecDeque<u32>,
  pub(crate) line_x: Byte,
  pub(crate) pushed_x: Byte,
  pub(crate) fetch_x: Byte,
  bgw_fetch_data: [Byte; 3],
  fetch_entry_data: [Byte; 6],
  pub(crate) map_y: Byte,
  pub(crate) map_x: Byte,
  pub(crate) tile_y: Byte,
  pub(crate) fifo_x: Byte,

  fetched_entry_count: Byte,
  fetched_entrys: [OamEntry; 3],
}

impl PixelFiFo {
  pub fn new() -> Self {
    Self {
      cur_fetch_state: FetchState::Tile,
      pixel_fifo: VecDeque::new(),
      line_x: 0,
      pushed_x: 0,
      fetch_x: 0,
      bgw_fetch_data: [0; 3],
      fetch_entry_data: [0; 6],
      map_y: 0,
      map_x: 0,
      tile_y: 0,
      fifo_x: 0,
      fetched_entry_count: 0,
      fetched_entrys: [OamEntry::default(); 3],
    }
  }

  pub fn fetch(&mut self, lcd: &mut Lcd, picture_bus: &PictureBus, window_line: Byte) {
    match self.cur_fetch_state {
      FetchState::Tile => {
        self.fetched_entry_count = 0;

        if lcd.bgw_enable() {
          self.bgw_fetch_data[0] = picture_bus
            .vram_read(lcd.bg_map_area() + (self.map_x as Address / 8 + self.map_y as Address / 8 * 32));
          if lcd.bgw_data_area() == 0x8800 {
            self.bgw_fetch_data[0] += 128;
          }

          self.load_window_tile(lcd, picture_bus, window_line);
        }

        // if lcd.obj_enable() && line_sprites {
        //   self.load_sprite_tile();
        // }

        self.cur_fetch_state = FetchState::Data0;
        self.fetch_x += 8;
      }
      FetchState::Data0 => {
        self.bgw_fetch_data[1] = picture_bus.vram_read(lcd.bgw_data_area() +( self.bgw_fetch_data[0] * 16 + self.tile_y) as Address);
        self.load_sprite_data(0, &lcd, &picture_bus);
        self.cur_fetch_state = FetchState::Data1;
      },
      FetchState::Data1 => {
        self.bgw_fetch_data[2] = picture_bus.vram_read(lcd.bgw_data_area() + (self.bgw_fetch_data[0] * 16 + self.tile_y + 1) as Address);
        self.load_sprite_data(1, &lcd, &picture_bus);
        self.cur_fetch_state = FetchState::Idle;
      },
      FetchState::Idle => {
        self.cur_fetch_state = FetchState::Push;
      },
      FetchState::Push => {
        if self.fifo_add(&lcd) {
          self.cur_fetch_state = FetchState::Tile;
        }
      },
    }
  }

  pub fn push_pixel(&mut self, lcd: &Lcd) -> Option<(Byte, u32)> {
    let mut ret = None;
    if self.pixel_fifo.len() > 8 {
      let pixel = self.pixel_fifo.pop_front().unwrap();
      
      if self.line_x >= lcd.scroll_x % 8 {
        // put video buffer
        ret = Some((self.pushed_x, pixel));
        self.pushed_x += 1;
      }

      self.line_x +=1;
    }
    ret
  }

  fn load_window_tile(&mut self, lcd: &Lcd, picture_bus: &PictureBus, window_line: Byte) {
    if !lcd.win_enable() {
      return;
    }

    let win_y = lcd.win_y;
    if is_between(self.fetch_x + 7, lcd.win_x, YRES + 14) && is_between(lcd.ly, win_y, XRES) {
      let w_tile_y = window_line / 8;

      self.bgw_fetch_data[0] = picture_bus.vram_read(
        lcd.win_map_area() + ((self.fetch_x + 7 - lcd.win_x) / 8 + w_tile_y * 32) as Address,
      );
      if lcd.bgw_data_area() == 0x8800 {
        self.bgw_fetch_data[0] += 128;
      }
    }
  }

  fn load_sprite_data(&mut self, offset: usize, lcd: &Lcd ,picture_bus: &PictureBus,) {
    let cur_y = lcd.ly;
    let sprite_height = lcd.obj_height();

    for i in 0..self.fetched_entry_count as usize {
      let mut ty = ((cur_y + 16) - self.fetched_entrys[i].y) * 2;

      if self.fetched_entrys[i].y_flip() {
        // flipped upside down
        ty = sprite_height * 2 -2  - ty;
      }

      let mut tile_index = self.fetched_entrys[i].title;

      if sprite_height == 16 {
        // remove last bit
        tile_index &= 0xFE;
      }

      self.fetch_entry_data[(i*2)+offset] = picture_bus.vram_read(0x8000 + (tile_index * 16 + ty) as Address + offset as Address);
    }
  }

  fn fifo_add(&mut self, lcd: &Lcd) -> bool {
    if self.pixel_fifo.len() > 8 {
      // fifo si full
      return false;
    }

    if self.fetch_x  < (8 -(lcd.scroll_x % 8)) {
      // Note: true or false??
      return true;
    }
    for i in 0..8 {
      let bit = 7 - i;
      let hi = bit_eq(self.bgw_fetch_data[1], 1<< bit) as Byte;
      let lo = bit_eq(self.bgw_fetch_data[2], 1<< bit) as Byte;
      let idx = hi | (lo << 1);
      let mut color = lcd.bg_colors[idx as usize];

      if lcd.bgw_enable() {
        color = lcd.bg_colors[0];
      }

      if lcd.obj_enable() {
        color = self.fetch_sprite_pixels(lcd, color, idx);
      }

      self.fifo_push(color);
      self.fifo_x += 1;
    }

    true
  }

  fn fetch_sprite_pixels(&self, lcd:&Lcd, p_color: u32, bg_color: Byte) -> u32 {
    let mut color= p_color;
    for i in 0..self.fetched_entry_count as usize {
      let sp_x = self.fetched_entrys[i].x - 8 + lcd.scroll_x % 8;
      if sp_x + 8 < self.fifo_x {
        // past pixel point aleady
        continue;
      }

      let offset = self.fifo_x.overflowing_sub(sp_x);
      if offset.1 || offset.0 > 7 {
        // out of bounds
        continue;
      }

      let mut bit = 7 - offset.0;
      if self.fetched_entrys[i].x_flip() {
        bit = offset.0;
      }

      let hi = bit_eq(self.fetch_entry_data[i * 2] ,1<<bit);
      let lo = bit_eq(self.fetch_entry_data[i*2 + 1], 1<<bit);

      if !hi && !lo {
        continue;
      }

      if !self.fetched_entrys[i].priority() || bg_color == 0 {
        let idx = (hi as Byte | ((lo as Byte )<< 1)) as usize;
        color = if self.fetched_entrys[i].dmg() {
          lcd.sp2_colors[idx]
        } else {
          lcd.sp1_colors[idx]
        }
      };
    }
    color
  }

  fn fifo_push(&mut self, color: u32) {
    self.pixel_fifo.push_back(color);
  }

  pub fn fifo_reset(&mut self) {
    self.pixel_fifo.clear();
  }
}
