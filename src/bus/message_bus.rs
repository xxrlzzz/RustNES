use image::RgbaImage;
use queues::*;

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt,
  PpuRender(RgbaImage),
}

#[derive(Default)]
pub struct MessageBus(Queue<Message>);

impl MessageBus {
  pub fn new() -> Self {
    Self { 0: Queue::new() }
  }

  pub fn push(&mut self, msg: Message) -> bool {
    self.0.add(msg).is_ok()
  }

  pub fn pop(&mut self) -> Option<Message> {
    self.0.remove().ok()
  }

  pub fn peek(&mut self) -> Option<Message> {
    self.0.peek().ok()
  }
}

impl Iterator for MessageBus {
  type Item = Message;
  fn next(&mut self) -> Option<Message> {
    let r = self.0.peek().ok();
    let _ = self.0.remove();
    r
  }
}
