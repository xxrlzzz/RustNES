use image::{ImageBuffer, Rgba};

pub type FrameBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RunningStatus {
  Running = 0,
  Pause = 1,
  LostFocus = 2,
  PauseAndLostFocus = 3,
}

impl From<u8> for RunningStatus {
  fn from(v: u8) -> Self {
    match v {
      0 => Self::Running,
      1 => Self::Pause,
      2 => Self::LostFocus,
      3 => Self::PauseAndLostFocus,
      _ => panic!(""),
    }
  }
}
impl RunningStatus {
  pub fn unpause(&mut self) {
    *self = Self::from(*self as u8 & Self::LostFocus as u8)
  }

  pub fn pause(&mut self) {
    *self = Self::from(*self as u8 | Self::Pause as u8)
  }

  pub fn is_pausing(&self) -> bool {
    (*self as u8 & Self::Pause as u8) != 0
  }

  pub fn unfocus(&mut self) {
    *self = Self::from(*self as u8 | Self::LostFocus as u8);
  }

  pub fn focus(&mut self) {
    *self = Self::from(*self as u8 & Self::Pause as u8);
  }

  pub fn is_focusing(&self) -> bool {
    (*self as u8 & Self::LostFocus as u8) == 0
  }
}

pub trait Instance {
  fn step(&mut self) -> u32;
  fn consume_message(&mut self);
  fn can_run(&self) -> bool;
  fn take_rgba(&mut self) -> Option<FrameBuffer>;
  fn stop(&mut self);

  // TODO: move out this
  fn do_save(&self, file: &String);

  fn focus(&mut self);
  fn unfocus(&mut self);
  fn is_pausing(&mut self) -> bool;
  fn toggle_pause(&mut self);
}
