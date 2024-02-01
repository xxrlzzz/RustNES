pub type Byte = u8;
pub type Address = u16;

#[inline]
pub fn bit_eq<T: std::ops::BitAnd<Output = T> + PartialEq + Copy>(a: T, b: T) -> bool {
  // let mut ca = a;
  // ca &= b;
  return (a & b) == b;
}

mod test {
  use crate::types::bit_eq;

  #[test]
  fn bit_eq_benchmark() {
    use std::time::Instant;

    let mut start = Instant::now();
    for i in 1..1788908 {
      bit_eq(1, i);
    }
    println!("{:?}", Instant::now() - start);
    start = Instant::now();
    for i in 1..1788908 {
      let _ = (1 & i) != 0;
    }
    println!("{:?}", Instant::now() - start);
  }


  #[derive(Debug)]
  #[allow(dead_code)]
  enum Foo {
    F1 = 1,
    F2,
    F3,
    F4 = 5,
    F5,
  }

  enum Boo {
    LOW,
    HIGH,
  }
  #[test]
  fn u8_test() {
    let i: u8 = 1;
    assert_eq!((i as u16) << 8, 1 << 8);
    let i: u16 = 1 << 8;
    assert_eq!(i as u8, 0);
    assert!(bit_eq(0u8.overflowing_sub(1).0, 1 << 7));
    assert!(0u8.overflowing_sub(1).1);
    let left: u8 = 1 << 7;
    let right: u8 = 1 << 7;
    assert!(bit_eq(0x100, (left as u16) + (right as u16)));
    println!("{} {}", Foo::F2 as u8, Foo::F5 as u8,);
    println!("{} {}", !0xFF00 as u16, !0x00FF as u16);
    println!("{} {}", Boo::LOW as u16, Boo::HIGH as u16);
    println!("{}", (0x2 as u8 & 0xe0) >> 5);
    println!("{}", (-1 % 8));
  }

  #[test]
  fn i8_test() {
    let offset = i8::from_be_bytes([0xfb]);
    let pc = 32783 as i32;
    assert_eq!((offset as i32).wrapping_add(pc), 32778);
  }
}
