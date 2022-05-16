use std::{collections::HashMap, time::Instant};

pub mod serializer;

pub type Byte = u8;
pub type Address = u16;

pub fn bit_eq<T: std::ops::BitAndAssign + PartialEq + Copy>(a: T, b: T) -> bool {
  let mut ca = a;
  ca &= b;
  return ca == b;
}

pub type MatrixType = HashMap<&'static str, u128>;

pub fn sample_profile(start: &mut Instant, category: &'static str, matrix: &mut MatrixType) {
  let now = Instant::now();
  let duration = (now - *start).as_micros();
  matrix
    .entry(category)
    .and_modify(|e| *e += duration)
    .or_insert(duration);
  *start = now;
}

mod test {

  #[test]
  fn bit_eq_benchmark() {
    use crate::common;
    use std::time::Instant;

    let mut start = Instant::now();
    for i in 1..1788908 {
      common::bit_eq(1, i);
    }
    println!("{:?}", Instant::now() - start);
    start = Instant::now();
    for i in 1..1788908 {
      let _ = (1 & i) != 0;
    }
    println!("{:?}", Instant::now() - start);
  }
}
