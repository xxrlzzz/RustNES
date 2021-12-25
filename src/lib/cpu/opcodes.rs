use crate::types::*;

pub const INSTRUCTION_MODE_MASK: Byte = 0x3;
pub const OPERATION_MASK: Byte = 0xe0;
pub const OPERATION_SHIFT: Byte = 5;

pub const ADDR_MODE_MASK: Byte = 0x1c;
pub const ADDR_MODE_SHIFT: Byte = 2;

pub const BRANCH_INSTRUCTION_MASK: Byte = 0x1f;
pub const BRANCH_INSTRUCTION_MASK_RESULT: Byte = 0x10;
pub const BRANCH_CONDITION_MASH: Byte = 0x20;
pub const BRANCH_ON_FLAG_SHIFT: Byte = 6;

pub const NMI_VECTOR: Address = 0xFFFA;
pub const RESET_VECTOR: Address = 0xFFFC;
pub const IRQ_VECTOR: Address = 0xFFFE;

pub mod branch_on_flag {
  pub const NEGATIVE: u8 = 0;
  pub const OVERFLOW: u8 = 1;
  pub const CARRY: u8 = 2;
  pub const ZERO: u8 = 3;
}

pub mod operation_implied {
  use crate::types::*;
  pub const NOP: Byte = 0xea;
  pub const BRK: Byte = 0x00;
  pub const JSR: Byte = 0x20;
  pub const RTI: Byte = 0x40;
  pub const RTS: Byte = 0x60;
  pub const JMP: Byte = 0x4c;
  pub const JMPI: Byte = 0x6c;
  pub const PHP: Byte = 0x08;
  pub const PLP: Byte = 0x28;
  pub const PHA: Byte = 0x48;
  pub const PLA: Byte = 0x68;
  pub const DEX: Byte = 0xca;
  pub const DEY: Byte = 0x88;
  pub const TAY: Byte = 0xa8;
  pub const INY: Byte = 0xc8;
  pub const INX: Byte = 0xe8;
  pub const CLC: Byte = 0x18;
  pub const SEC: Byte = 0x38;
  pub const CLI: Byte = 0x58;
  pub const SEI: Byte = 0x78;
  pub const TYA: Byte = 0x98;
  pub const CLV: Byte = 0xb8;
  pub const CLD: Byte = 0xd8;
  pub const SED: Byte = 0xf8;
  pub const TXA: Byte = 0x8a;
  pub const TXS: Byte = 0x9a;
  pub const TAX: Byte = 0xaa;
  pub const TSX: Byte = 0xba;
}

pub mod operation1 {
  pub const ORA: u8 = 0;
  pub const AND: u8 = 1;
  pub const EOR: u8 = 2;
  pub const ADC: u8 = 3;
  pub const STA: u8 = 4;
  pub const LDA: u8 = 5;
  pub const CMP: u8 = 6;
  pub const SBC: u8 = 7;
}

pub mod addr_mode1 {
  pub const INDEXED_INDIRECT_X: u8 = 0;
  pub const ZERO_PAGE: u8 = 1;
  pub const IMMEDIATE: u8 = 2;
  pub const ABSOLUTE: u8 = 3;
  pub const INDIRECT_Y: u8 = 4;
  pub const INDEXED_X: u8 = 5;
  pub const ABSOLUTE_Y: u8 = 6;
  pub const ABSOLUTE_X: u8 = 7;
}

pub mod operation2 {
  pub const ASL: u8 = 0;
  pub const ROL: u8 = 1;
  pub const LSR: u8 = 2;
  pub const ROR: u8 = 3;
  pub const STX: u8 = 4;
  pub const LDX: u8 = 5;
  pub const DEC: u8 = 6;
  pub const INC: u8 = 7;
}

pub mod addr_mode2 {
  pub const IMMEDIATE: u8 = 0;
  pub const ZERO_PAGE: u8 = 1;
  pub const ACCUMULATOR: u8 = 2;
  pub const ABSOLUTE: u8 = 3;
  pub const INDEXED: u8 = 5;
  pub const ABSOLUTE_INDEXED: u8 = 7;
}

pub mod operation0 {
  pub const BIT: u8 = 1;
  pub const STY: u8 = 4;
  pub const LDY: u8 = 5;
  pub const CPY: u8 = 6;
  pub const CPX: u8 = 7;
}

#[rustfmt::skip]
pub const OPERATION_CYCLES: [Byte; 0x100] = [
  7, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 0, 4, 6, 0,
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
  6, 6, 0, 0, 3, 3, 5, 0, 4, 2, 2, 0, 4, 4, 6, 0, 
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
  6, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 3, 4, 6, 0,
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
  6, 6, 0, 0, 0, 3, 5, 0, 4, 2, 2, 0, 5, 4, 6, 0,
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
  0, 6, 0, 0, 3, 3, 3, 0, 2, 0, 2, 0, 4, 4, 4, 0, 
  2, 6, 0, 0, 4, 4, 4, 0, 2, 5, 2, 0, 0, 5, 0, 0,
  2, 6, 2, 0, 3, 3, 3, 0, 2, 2, 2, 0, 4, 4, 4, 0, 
  2, 5, 0, 0, 4, 4, 4, 0, 2, 4, 2, 0, 4, 4, 4, 0,
  2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 0, 4, 4, 6, 0, 
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
  2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 2, 4, 4, 6, 0, 
  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
];
