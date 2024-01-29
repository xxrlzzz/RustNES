use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

pub trait Filter {
  fn step(&mut self, x: f32) -> f32;
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SoundFilter {
  b0: f32,
  b1: f32,
  a1: f32,

  prev_x: f32,
  prev_y: f32,
}

impl SoundFilter {
  pub(crate) fn new_low_pass_filter(sample_rate: f32, cut_off_freq: f32) -> Self {
    let c = sample_rate / PI / cut_off_freq;
    let a0i = 1.0 / (1.0 + c);

    Self {
      b0: a0i,
      b1: a0i,
      a1: (1. - c) * a0i,

      prev_x: 0.,
      prev_y: 0.,
    }
  }

  pub(crate) fn new_high_pass_filter(sample_rate: f32, cut_off_freq: f32) -> Self {
    let c = sample_rate / PI / cut_off_freq;
    let a0i = 1.0 / (1.0 + c);

    Self {
      b0: c * a0i,
      b1: -c * a0i,
      a1: (1. - c) * a0i,

      prev_x: 0.,
      prev_y: 0.,
    }
  }
}

impl Filter for SoundFilter {
  fn step(&mut self, x: f32) -> f32 {
    let y = self.b0 * x + self.b1 * self.prev_x - self.a1 * self.prev_y;
    self.prev_x = x;
    self.prev_y = y;
    y
  }
}

pub(crate) type SoundFilterChain = Vec<SoundFilter>;

impl Filter for SoundFilterChain {
  fn step(&mut self, x: f32) -> f32 {
    let mut y = x;
    for filter in self.iter_mut() {
      y = filter.step(y);
    }
    y
  }
}
