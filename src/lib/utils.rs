// use crate::types::*;

// trait TyEq {}

// impl<T> TyEq for (T, T) {}

// trait BitCompare {}
// impl<T> BitCompare for T
// where
//   T: std::ops::BitAnd,
//   T::Output: std::ops::BitAnd,
// {
// }

pub fn bit_eq<T: std::ops::BitAndAssign + PartialEq + Copy>(a: T, b: T) -> bool {
  let mut ca = a;
  ca &= b;
  return ca == b;
}

// pub fn bit_eq(a: Byte, b: Byte) -> bool {
//   return (a & b) == b;
// }

// pub fn bit_eq_16(a: Address, b: Address) -> bool {
//   return (a & b) == b;
// }
