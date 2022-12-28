use image::RgbaImage;

use crate::cpu::InterruptType;

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt(InterruptType),
  PpuRender(RgbaImage),
}
