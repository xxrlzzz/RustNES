use queues::*;
use sfml::graphics::Color;

#[derive(Debug, Clone)]
pub enum Message {
  CpuInterrupt,
  PpuRender(Vec<Vec<Color>>),
}

pub struct MessageBus {
  queue: Queue<Message>,
}

impl MessageBus {
  pub fn new() -> Self {
    Self {
      queue: Queue::new(),
    }
  }

  pub fn push(&mut self, msg: Message) -> bool {
    match self.queue.add(msg) {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn pop(&mut self) -> Option<Message> {
    let front = self.queue.remove();
    match front {
      Ok(msg) => Some(msg),
      Err(_) => None,
    }
  }

  pub fn peek(&mut self) -> Option<Message> {
    let front = self.queue.peek();
    match front {
      Ok(msg) => Some(msg),
      Err(_) => None,
    }
  }
}
