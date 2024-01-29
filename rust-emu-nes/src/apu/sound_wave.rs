use serde::{Deserialize, Serialize};

use rust_emu_common::types::*;

use super::ReadCallback;

const LENGTH_TABLE: [Byte; 32] = [
  0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x80, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
  0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

const DUTY_TABLE: [[Byte; 8]; 4] = [
  [0, 1, 0, 0, 0, 0, 0, 0],
  [0, 1, 1, 0, 0, 0, 0, 0],
  [0, 1, 1, 1, 1, 0, 0, 0],
  [1, 0, 0, 1, 1, 1, 1, 1],
];

#[rustfmt::skip]
const TRIANGLE_TABLE: [Byte; 32] = [
  15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 
  0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

const NOISE_TABLE: [Address; 16] = [
  4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const DMC_TABLE: [Byte; 16] = [
  214, 190, 170, 160, 143, 127, 113, 107, 95, 80, 71, 64, 53, 42, 36, 27,
];

#[derive(Default, Serialize, Deserialize)]
struct Envelope {
  enabled: bool,
  length_enabled: bool,
  length_value: Byte,
  envelop_enabled: bool,
  envelop_loop: bool,
  envelop_start: bool,
  envelop_period: Byte,
  envelop_value: Byte,
  envelop_volume: Byte,
  constant_volume: Byte,
}

impl Envelope {
  pub(crate) fn new() -> Envelope {
    Envelope {
      enabled: false,
      length_enabled: false,
      length_value: 0,
      envelop_enabled: false,
      envelop_loop: false,
      envelop_start: false,
      envelop_period: 0,
      envelop_value: 0,
      envelop_volume: 0,
      constant_volume: 0,
    }
  }

  pub fn length_value(&self) -> Byte {
    self.length_value
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.enabled = value;
    if !value {
      self.length_value = 0;
    }
  }

  fn output(&self) -> Byte {
    if !self.enabled {
      return 0;
    }
    if self.length_value == 0 {
      return 0;
    }
    if self.envelop_enabled {
      self.envelop_volume
    } else {
      self.constant_volume
    }
  }

  pub(crate) fn step_length(&mut self) {
    if self.length_enabled && self.length_value > 0 {
      self.length_value -= 1;
    }
  }

  pub(crate) fn step_envelope(&mut self) {
    if self.envelop_start {
      self.envelop_volume = 15;
      self.envelop_value = self.envelop_period;
      self.envelop_start = false;
    } else if self.envelop_value > 0 {
      self.envelop_value -= 1;
    } else {
      if self.envelop_volume > 0 {
        self.envelop_volume -= 1;
      } else if self.envelop_loop {
        self.envelop_volume = 15;
      }
      self.envelop_value = self.envelop_period;
    }
  }

  pub(crate) fn write_length(&mut self, value: Byte) {
    self.length_value = LENGTH_TABLE[(value >> 3) as usize];
    self.envelop_start = true;
  }

  pub(crate) fn write_control(&mut self, value: Byte) {
    self.length_enabled = !bit_eq(value, 0x20);
    self.envelop_loop = !self.length_enabled;
    self.envelop_enabled = !bit_eq(value, 0x10);
    self.envelop_period = value & 0x0F;
    self.constant_volume = value & 0x0F;
    self.envelop_start = true;
  }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Pulse {
  channel: Byte,
  sweep_reload: bool,
  sweep_enabled: bool,
  sweep_negate: bool,
  sweep_period: Byte,
  sweep_value: Byte,
  sweep_shift: Byte,
  timer_period: Address,
  timer_value: Address,
  duty_mode: Byte,
  duty_value: Byte,
  envelope: Envelope,
}

impl Pulse {
  pub(crate) fn new(channel: Byte) -> Self {
    Self {
      channel: channel,
      sweep_reload: false,
      sweep_enabled: false,
      sweep_negate: false,
      sweep_period: 0,
      sweep_value: 0,
      sweep_shift: 0,
      timer_period: 0,
      timer_value: 0,
      duty_mode: 0,
      duty_value: 0,
      envelope: Envelope::new(),
    }
  }

  pub fn length_value(&self) -> Byte {
    self.envelope.length_value()
  }

  pub(crate) fn output(&self) -> Byte {
    if DUTY_TABLE[self.duty_mode as usize][self.duty_value as usize] == 0 {
      return 0;
    }
    if self.timer_period < 8 || self.timer_period > 0x7FF {
      return 0;
    }
    return self.envelope.output();
  }

  pub(crate) fn set_enabled(&mut self, enable: bool) {
    self.envelope.set_enabled(enable);
  }

  fn sweep(&mut self) {
    let delta = self.timer_period >> self.sweep_shift;
    if self.sweep_negate {
      self.timer_period = self.timer_period.wrapping_sub(delta);
      if self.channel == 1 {
        self.timer_period = self.timer_period.wrapping_sub(1);
      }
    } else {
      self.timer_period = self.timer_period.wrapping_add(delta);
    }
  }

  pub(super) fn step_length(&mut self) {
    self.envelope.step_length();
  }

  pub(super) fn step_sweep(&mut self) {
    if self.sweep_reload {
      if self.sweep_enabled && self.sweep_value == 0 {
        self.sweep();
      }
      self.sweep_value = self.sweep_period;
      self.sweep_reload = false;
    } else if self.sweep_value > 0 {
      self.sweep_value -= 1;
    } else {
      if self.sweep_enabled {
        self.sweep();
      }
      self.sweep_value = self.sweep_period;
    }
  }

  pub(super) fn step_envelope(&mut self) {
    self.envelope.step_envelope();
  }

  pub(crate) fn step_timer(&mut self) {
    if self.timer_value > 0 {
      self.timer_value -= 1;
    } else {
      self.timer_value = self.timer_period;
      self.duty_value = (self.duty_value + 1) % 8;
    }
  }

  pub(crate) fn write_timer_high(&mut self, value: Byte) {
    self.envelope.write_length(value);
    self.timer_period = (self.timer_period & 0xFF) | ((value as Address & 7) << 8);
    self.duty_value = 0;
  }

  pub(crate) fn write_timer_low(&mut self, value: Byte) {
    self.timer_period = (self.timer_period & 0xFF00) | value as Address;
  }

  pub(crate) fn write_sweep(&mut self, value: Byte) {
    self.sweep_enabled = bit_eq(value >> 7, 1);
    self.sweep_period = (value >> 4 & 7) + 1;
    self.sweep_negate = bit_eq(value >> 3, 1);
    self.sweep_shift = value & 7;
    self.sweep_reload = true;
  }

  pub(crate) fn write_control(&mut self, value: Byte) {
    self.duty_mode = (value >> 6) & 3;
    self.envelope.write_control(value);
  }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Noise {
  timer_period: Address,
  timer_value: Address,
  timer_mode: bool,
  shift_register: Address,
  envelope: Envelope,
}

impl Noise {
  pub(crate) fn new() -> Self {
    Self {
      timer_period: 0,
      timer_value: 0,
      shift_register: 1,
      timer_mode: false,
      envelope: Envelope::new(),
    }
  }

  pub(crate) fn length_value(&self) -> Byte {
    self.envelope.length_value()
  }

  pub(crate) fn set_enabled(&mut self, enable: bool) {
    self.envelope.set_enabled(enable);
  }

  pub(crate) fn write_control(&mut self, value: Byte) {
    self.envelope.write_control(value);
  }
  pub(super) fn step_envelope(&mut self) {
    self.envelope.step_envelope();
  }

  pub(super) fn step_length(&mut self) {
    self.envelope.step_length();
  }

  pub(crate) fn write_length(&mut self, value: Byte) {
    self.envelope.write_length(value);
  }

  pub(crate) fn output(&self) -> Byte {
    if bit_eq(self.shift_register, 1) {
      return 0;
    }
    return self.envelope.output();
  }

  pub(super) fn step_timer(&mut self) {
    if self.timer_value != 0 {
      self.timer_value -= 1;
      return;
    }
    // Reset timer and shift register.
    self.timer_value = self.timer_period;
    let shift = if self.timer_mode { 6 } else { 1 };
    let b1 = self.shift_register & 1;
    let b2 = (self.shift_register >> shift) & 1;
    self.shift_register = ((b1 ^ b2) << 14) | (self.shift_register >> 1);
  }

  pub(crate) fn write_period(&mut self, value: Byte) {
    // m--- iiii       mode, period index
    self.timer_mode = bit_eq(value, 0x80);
    self.timer_period = NOISE_TABLE[value as usize & 0x0F];
  }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Triangle {
  enabled: bool,
  length_enabled: bool,
  pub length_value: Byte,
  timer_period: Address,
  timer_value: Address,
  duty_value: Byte,
  counter_period: Byte,
  counter_value: Byte,
  counter_reload: bool,
}

impl Triangle {
  pub(crate) fn new() -> Self {
    Self {
      enabled: false,
      length_enabled: false,
      length_value: 0,
      timer_period: 0,
      timer_value: 0,
      duty_value: 0,
      counter_period: 0,
      counter_value: 0,
      counter_reload: false,
    }
  }

  pub(crate) fn set_enabled(&mut self, enable: bool) {
    self.enabled = enable;
    if !enable {
      self.length_value = 0;
    }
  }

  pub(crate) fn output(&self) -> Byte {
    if !self.enabled {
      return 0;
    }
    if self.timer_period < 3 {
      return 0;
    }
    if self.length_value == 0 {
      return 0;
    }
    if self.counter_value == 0 {
      return 0;
    }
    TRIANGLE_TABLE[self.duty_value as usize]
  }

  pub(crate) fn step_counter(&mut self) {
    if self.counter_reload {
      self.counter_value = self.counter_period;
    } else if self.counter_value > 0 {
      self.counter_value -= 1;
    }
    if self.length_enabled {
      self.counter_reload = false;
    }
  }

  pub(crate) fn step_length(&mut self) {
    if self.length_enabled && self.length_value > 0 {
      self.length_value -= 1;
    }
  }

  pub(super) fn step_timer(&mut self) {
    if self.timer_value == 0 {
      self.timer_value = self.timer_period;
      if self.counter_value > 0 && self.counter_value > 0 {
        self.duty_value = (self.duty_value + 1) % 32;
      }
    } else {
      self.timer_value -= 1;
    }
  }

  pub(crate) fn write_timer_high(&mut self, value: Byte) {
    self.length_value = LENGTH_TABLE[value as usize >> 3];
    self.timer_period = (self.timer_period & 0xFF) | ((value as Address & 7) << 8);
    self.timer_value = self.timer_period;
    self.counter_reload = true;
  }

  pub(crate) fn write_timer_low(&mut self, value: Byte) {
    self.timer_period = (self.timer_period & 0xFF00) | value as Address;
  }

  pub(crate) fn write_control(&mut self, value: Byte) {
    self.length_enabled = !bit_eq(value, 0x80);
    self.counter_period = value & 0x7F;
  }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct DMC {
  enabled: bool,
  value: Byte,
  sample_address: Address,
  sample_length: Address,
  current_address: Address,
  pub current_length: Address,
  shift_register: Byte,
  bit_count_: Byte,
  tick_period: Byte,
  tick_value: Byte,
  loop_enable: bool,
  irq: bool,
  // cpu
  #[serde(skip)]
  read_cb: Option<ReadCallback>,
}

impl DMC {
  pub(crate) fn new() -> Self {
    Self {
      enabled: false,
      value: 0,
      sample_address: 0,
      sample_length: 0,
      current_address: 0,
      current_length: 0,
      shift_register: 0,
      bit_count_: 0,
      tick_period: 0,
      tick_value: 0,
      loop_enable: false,
      irq: false,
      read_cb: None,
    }
  }

  pub fn set_read_cb(&mut self, read_cb: ReadCallback) {
    self.read_cb = Some(read_cb);
  }

  pub(crate) fn set_enabled(&mut self, enable: bool) {
    self.enabled = enable;
    if !enable {
      self.current_length = 0;
    } else if self.current_length == 0 {
      self.restart();
    }
  }

  pub(crate) fn output(&self) -> Byte {
    return self.value;
  }

  pub(crate) fn step_shifter(&mut self) {
    if self.bit_count_ == 0 {
      return;
    }

    if bit_eq(self.shift_register, 1) {
      if self.value <= 125 {
        self.value += 2;
      }
    } else {
      if self.value >= 2 {
        self.value -= 2;
      }
    }
    self.shift_register >>= 1;
    self.bit_count_ -= 1;
  }

  pub(crate) fn step_reader(&mut self) {
    if self.current_length <= 0 || self.bit_count_ != 0 {
      return;
    }
    let val = self.read_cb.as_mut().unwrap()(self.current_address);
    self.shift_register = val;
    // self.cpu.skipDMCCycles();
    // self.shift_register = self.cpu.read(self.current_address);
    self.bit_count_ = 8;
    self.current_address += 1;
    if self.current_address == 0 {
      self.current_address = 0x8000;
    }
    self.current_address -= 1;
    if self.current_length == 0 && self.loop_enable {
      self.restart();
    }
  }

  pub(super) fn step_timer(&mut self) {
    if !self.enabled {
      return;
    }
    self.step_reader();
    if self.tick_value == 0 {
      self.tick_value = self.tick_period;
      self.step_shifter();
    } else {
      self.tick_value -= 1;
    }
  }

  fn restart(&mut self) {
    self.current_address = self.sample_address;
    self.current_length = self.sample_length;
  }

  pub(crate) fn write_length(&mut self, value: Address) {
    self.sample_length = (value << 4) | 1;
  }

  pub(crate) fn write_address(&mut self, value: Address) {
    self.sample_address = (value << 6) | 0xC000;
  }

  pub(crate) fn write_value(&mut self, value: Byte) {
    self.value = value & 0x7F;
  }

  pub(crate) fn write_control(&mut self, value: Byte) {
    self.irq = bit_eq(value, 0x80);
    self.loop_enable = bit_eq(value, 0x40);
    self.tick_period = DMC_TABLE[value as usize & 0xF];
  }
}
