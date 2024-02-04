use crate::types::*;


pub trait RegisterHandler {
  fn read(&mut self, address: Address) -> Option<Byte>;
  fn write(&mut self, address: Address, value: Byte) -> bool;
  fn dma(&mut self, page: *const Byte) -> bool;
}

pub trait MainBus {
  fn read(&mut self, addr: Address) -> Byte;
  fn write(&mut self, addr: Address, value: Byte);
}