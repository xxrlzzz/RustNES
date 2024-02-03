use std::sync::mpsc;

use serde::{Deserialize, Serialize};
use rust_emu_common::types::*;

use crate::{bus::RegisterHandler, instance::Message, interrupt};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Timer {
  div: Address,
  tima: Byte,
  tma: Byte,
  tac: Byte,

  #[serde(skip)]
  message_sx: Option<mpsc::Sender<Message>>, // interrupt channel
}

impl Timer {
  pub fn new(message_sx: mpsc::Sender<Message>) -> Self {
    Self {
      message_sx: Some(message_sx),
      div: 0xABCC,
      ..Default::default()
    }
  }
  pub(crate) fn step(&mut self) {
    let prev_div = self.div;

    self.div.overflowing_add(1).0;

    let timer_update = match self.tac & 0x3 {
      0x0 => bit_eq(prev_div, 1 << 9) && !bit_eq(self.div, 1 << 9),
      0x1 => bit_eq(prev_div, 1 << 3) && !bit_eq(self.div, 1 << 3),
      0x2 => bit_eq(prev_div, 1 << 5) && !bit_eq(self.div, 1 << 5),
      0x3 => bit_eq(prev_div, 1 << 7) && !bit_eq(self.div, 1 << 7),
      _ => false,
    };
    if timer_update && bit_eq(self.tac, 1 << 2) {
      self.tima += 1;
      if self.tima == 0xFF {
        self.tima = self.tma;
        self.message_sx.as_ref().unwrap().send(Message::CpuInterrupt(interrupt::INT_TIMER)).unwrap();
      }
    }
  }
}

impl RegisterHandler for Timer {
  fn read(&mut self, address: Address) -> Option<Byte> {
    match address {
      0xFF04 => Some((self.div >> 8) as Byte),
      0xFF05 => Some(self.tima),
      0xFF06 => Some(self.tma),
      0xFF07 => Some(self.tac),
      _ => None,
    }
  }

  fn write(&mut self, address: Address, value: Byte) -> bool {
    match address {
      0xFF04 => {
        self.tima = value;
        true
      }
      0xFF05 => {
        self.tma = value;
        true
      }
      0xFF06 => {
        self.tac = value;
        true
      }
      0xFF07 => {
        self.tac = value;
        true
      }
      _ => false,
    }
  }

  fn dma(&mut self, page: *const Byte) -> bool {
    false
  }
}
