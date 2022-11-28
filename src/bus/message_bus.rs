use image::RgbaImage;

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt,
  PpuRender(RgbaImage),
}
