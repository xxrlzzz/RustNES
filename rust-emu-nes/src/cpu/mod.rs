use std::convert::Into;

use rust_emu_common::types::*;

use self::opcodes::*;
use crate::bus::main_bus::MainBus;
use log::warn;
use serde::Deserialize;
use serde::Serialize;

mod opcodes;

#[derive(Default, PartialEq, Debug, Clone)]
pub enum InterruptType {
  IRQ,
  NMI,
  BRK,
  #[default]
  None,
}

// 256 read + 256 write + 1 dummy read
const DMA_CYCLES: u32 = 513;
const DMC_CYCLES: u32 = 4;
const INTERRUPT_CYCLES: u32 = 7;

mod flag_const {
  use rust_emu_common::types::*;
  // 7 6 5 4 3 2 1 0
  // N V - B D I Z C

  pub const NEGATIVE: Byte = 1 << 7;
  pub const OVERFLOW: Byte = 1 << 6;
  pub const DECIMAL: Byte = 1 << 3;
  pub const INTERRUPT: Byte = 1 << 2;
  pub const ZERO: Byte = 1 << 1;
  pub const CARRY: Byte = 1;
  pub const ALL: Byte = NEGATIVE | OVERFLOW | (1 << 5) | DECIMAL | INTERRUPT | ZERO | CARRY;
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Flag(Byte);

impl Flag {
  pub fn clear(&mut self) {
    self.0 = 1 << 5;
  }
  pub fn set_at(&mut self, pos: Byte, v: bool) {
    if v {
      self.0 |= pos;
    } else {
      self.0 &= flag_const::ALL - pos;
    }
  }

  pub fn get_at(&self, pos: Byte) -> bool {
    bit_eq(self.0, pos)
  }

  pub fn set_by_check(&mut self, pos: Byte, v: Byte) {
    self.set_at(pos, bit_eq(v, pos));
  }

  pub fn set_all(&mut self, v: Byte) {
    self.0 = v | (1 << 5);
  }
}

impl Into<Byte> for Flag {
  fn into(self) -> Byte {
    return self.0;
  }
}
#[derive(Serialize, Deserialize)]
pub struct Cpu {
  skip_cycles: u32,
  cycles: u32,

  // registers
  r_pc: Address, // program counter
  r_sp: Byte,    // stack pointer
  r_a: Byte,     // accumulator
  r_x: Byte,     // index register, for offset of index and address.
  r_y: Byte,     // index register, only useful when LDX/STX.

  // status flag.
  flag: Flag,
  #[serde(skip)]
  interrupt: InterruptType,
  #[serde(skip)]
  main_bus: MainBus,
}

impl Cpu {
  pub fn new(main_bus: MainBus) -> Self {
    Self {
      skip_cycles: 0,
      cycles: 0,
      r_pc: 0,
      r_sp: 0,
      r_a: 0,
      r_x: 0,
      r_y: 0,
      flag: Flag(0),
      main_bus,
      interrupt: InterruptType::None,
    }
  }

  pub fn set_main_bus(&mut self, main_bus: MainBus) {
    self.main_bus = main_bus;
  }

  pub fn main_bus(&self) -> &MainBus {
    &self.main_bus
  }

  pub fn main_bus_mut(&mut self) -> &mut MainBus {
    &mut self.main_bus
  }

  pub fn reset(&mut self) {
    let reset_vector = self.read_address(opcodes::RESET_VECTOR);
    self.reset_at(reset_vector)
  }

  fn reset_at(&mut self, start_addr: Address) {
    self.cycles = 0;
    self.skip_cycles = 0;
    self.r_a = 0;
    self.r_x = 0;
    self.r_y = 0;
    self.flag.clear();
    self.flag.set_at(flag_const::INTERRUPT, true);
    self.r_pc = start_addr;
    // documented startup state
    self.r_sp = 0xFD;
  }

  fn get_flag(&self) -> Byte {
    return self.flag.into();
  }

  #[allow(dead_code)]
  pub(crate) fn print_flag(&self) {
    let psw = self.get_flag();
    log::debug!(
      "[CPU-STATUS] {:#x}:{:#x} A:{:#x}, X:{:#x}, Y:{:#x}, P:{:#x}, SP:{:#x}, CYC:{}",
      self.r_pc,
      self.main_bus.save_read(self.r_pc),
      self.r_a,
      self.r_x,
      self.r_y,
      psw,
      self.r_sp,
      (self.cycles - 1) * 3 % crate::ppu::SCANLINE_END_CYCLE_LENGTH
    );
  }

  #[inline]
  pub fn trigger_interrupt(&mut self, i_type: InterruptType) {
    if self.flag.get_at(flag_const::INTERRUPT)
      && i_type != InterruptType::NMI
      && i_type != InterruptType::BRK
    {
      return;
    }
    self.interrupt = i_type;
  }

  pub fn interrupt(&mut self, i_type: InterruptType) {
    if i_type == InterruptType::BRK {
      self.r_pc += 1;
    }

    self.push_stack((self.r_pc >> 8) as u8);
    self.push_stack(self.r_pc as u8);

    let flags = self.get_flag()
      + if i_type == InterruptType::BRK {
        1 << 4
      } else {
        0
      };
    self.push_stack(flags);

    self.flag.set_at(flag_const::INTERRUPT, true);

    self.r_pc = self.read_address(match i_type {
      InterruptType::IRQ => opcodes::IRQ_VECTOR,
      InterruptType::BRK => opcodes::IRQ_VECTOR,
      InterruptType::NMI => opcodes::NMI_VECTOR,
      _ => panic!("invalid interrupt type"),
    });

    self.skip_cycles += INTERRUPT_CYCLES;
  }

  #[inline]
  fn push_stack(&mut self, value: Byte) {
    self.main_bus.write(0x100 | self.r_sp as Address, value);
    // Hardware stacks grow downward!
    self.r_sp -= 1;
  }

  #[inline]
  fn pull_stack(&mut self) -> Byte {
    self.r_sp += 1;
    self.main_bus.read(0x100 | self.r_sp as Address)
  }

  #[inline]
  fn pull_stack_16(&mut self) -> Address {
    return self.pull_stack() as Address | (self.pull_stack() as Address) << 8;
  }

  #[inline]
  fn set_zn(&mut self, value: Byte) {
    self.flag.set_at(flag_const::ZERO, value == 0);
    self
      .flag
      .set_at(flag_const::NEGATIVE, bit_eq(value, flag_const::NEGATIVE));
  }

  #[inline]
  fn set_page_crossed(&mut self, addr_a: Address, addr_b: Address, inc: u32) {
    if (addr_a & 0xFF00) != (addr_b & 0xFF00) {
      self.skip_cycles += inc;
    }
  }

  #[inline]
  pub fn skip_dma_cycles(&mut self) {
    self.skip_cycles += DMA_CYCLES;
    // +1 if on add cycle
    self.skip_cycles += self.cycles & 1;
  }

  #[inline]
  pub fn skip_dmc_cycles(&mut self) {
    self.skip_cycles += DMC_CYCLES;
  }

  #[inline]
  pub fn reset_skip_cycles(&mut self) {
    // TODO: can we add skip cycle onto cycle?
    self.cycles += self.skip_cycles;
    self.skip_cycles = 0;
  }

  #[inline]
  fn read_address(&mut self, addr: Address) -> Address {
    self.main_bus.read_addr(addr) | self.main_bus.read_addr(addr + 1) << 8
  }

  #[inline]
  fn read_and_forward_pc(&mut self) -> Address {
    let res = self.main_bus.read_addr(self.r_pc);
    self.r_pc += 1;
    res
  }

  pub fn step(&mut self) -> u32 {
    self.cycles += 1;
    if self.skip_cycles > 0 {
      return self.skip_cycles;
    }

    if self.interrupt != InterruptType::None {
      self.interrupt(self.interrupt.clone());
      self.interrupt = InterruptType::None;
    }

    let opcode = self.read_and_forward_pc() as Byte;

    let cycle_length = opcodes::OPERATION_CYCLES[opcode as usize];
    // Using short-circuit evaluation, call the other function only if the first
    // failed ExecuteImplied must be called first and ExecuteBranch must be before
    // ExecuteType0
    if cycle_length != 0
      && (self.execute_implied(opcode)
        || self.execute_branch(opcode)
        || self.execute_type1(opcode)
        || self.execute_type2(opcode)
        || self.execute_type0(opcode))
    {
      self.skip_cycles += cycle_length as u32;
      if self.main_bus.check_and_reset_dma() {
        self.skip_dma_cycles();
      }
    } else {
      if opcode != 0xff {
        warn!("Unrecognized opcode {:#x}", opcode);
      }
    }
    self.skip_cycles
  }

  fn execute_implied(&mut self, opcode: Byte) -> bool {
    match opcode {
      operation_implied::NOP => (),
      operation_implied::BRK => self.trigger_interrupt(InterruptType::BRK),
      operation_implied::JSR => {
        // Push address of next instruction - 1 ,thus r_PC + 1 instead of r_PC + 2
        // since r_PC and r_PC + 1 are address of subroutine
        self.push_stack(((self.r_pc + 1) >> 8) as Byte);
        self.push_stack((self.r_pc + 1) as Byte);
        self.r_pc = self.read_address(self.r_pc);
      }
      operation_implied::RTS => {
        self.r_pc = self.pull_stack_16() + 1;
      }
      operation_implied::RTI => {
        let flag = self.pull_stack();
        self.flag.set_all(flag);
        self.r_pc = self.pull_stack_16();
      }
      operation_implied::JMP => self.r_pc = self.read_address(self.r_pc),
      operation_implied::JMPI => {
        let location = self.read_address(self.r_pc);

        // 6502 has a bug such that the when the vector of an indirect address
        // begins at the last byte of a page, the second byte is fetched from the
        // beginning of that page rather than the beginning of the next Recreating
        // here:
        let page = location & 0xFF00;
        self.r_pc = self.main_bus.read_addr(location);
        self.r_pc |= self.main_bus.read_addr(page | ((location + 1) & 0xFF)) << 8;
      }
      operation_implied::PHP => self.push_stack(self.get_flag() | (1 << 4)),
      operation_implied::PLP => {
        let flag = self.pull_stack();
        self.flag.set_all(flag);
      }
      operation_implied::PHA => self.push_stack(self.r_a),
      operation_implied::PLA => {
        self.r_a = self.pull_stack();
        self.set_zn(self.r_a);
      }
      operation_implied::DEX => {
        self.r_x = self.r_x.wrapping_sub(1);
        self.set_zn(self.r_x);
      }
      operation_implied::DEY => {
        self.r_y = self.r_y.wrapping_sub(1);
        self.set_zn(self.r_y);
      }
      operation_implied::TAY => {
        self.r_y = self.r_a;
        self.set_zn(self.r_y);
      }
      operation_implied::TAX => {
        self.r_x = self.r_a;
        self.set_zn(self.r_x);
      }
      operation_implied::TYA => {
        self.r_a = self.r_y;
        self.set_zn(self.r_a);
      }
      operation_implied::TXA => {
        self.r_a = self.r_x;
        self.set_zn(self.r_a);
      }
      operation_implied::TXS => {
        self.r_sp = self.r_x;
      }
      operation_implied::TSX => {
        self.r_x = self.r_sp;
        self.set_zn(self.r_x);
      }
      operation_implied::INX => {
        self.r_x = self.r_x.wrapping_add(1);
        self.set_zn(self.r_x);
      }
      operation_implied::INY => {
        self.r_y = self.r_y.wrapping_add(1);
        self.set_zn(self.r_y);
      }
      operation_implied::CLC => self.flag.set_at(flag_const::CARRY, false),
      operation_implied::SEC => self.flag.set_at(flag_const::CARRY, true),
      operation_implied::CLI => self.flag.set_at(flag_const::INTERRUPT, false),
      operation_implied::SEI => self.flag.set_at(flag_const::INTERRUPT, true),
      operation_implied::CLD => self.flag.set_at(flag_const::DECIMAL, false),
      operation_implied::SED => self.flag.set_at(flag_const::DECIMAL, true),
      operation_implied::CLV => self.flag.set_at(flag_const::OVERFLOW, false),
      _ => return false,
    };
    true
  }

  fn execute_branch(&mut self, opcode: Byte) -> bool {
    if opcode & BRANCH_INSTRUCTION_MASK != BRANCH_INSTRUCTION_MASK_RESULT {
      return false;
    }
    // branch is initialized to the condition required ( for the flag specified
    // later)
    let mut branch = bit_eq(opcode, BRANCH_CONDITION_MASH);
    // set branch to true if the given condition is met by the given flag
    // We use xor here, it is true if either both operands are true or false
    match opcode >> BRANCH_ON_FLAG_SHIFT {
      branch_on_flag::NEGATIVE => branch = !(branch ^ self.flag.get_at(flag_const::NEGATIVE)),
      branch_on_flag::OVERFLOW => branch = !(branch ^ self.flag.get_at(flag_const::OVERFLOW)),
      branch_on_flag::CARRY => branch = !(branch ^ self.flag.get_at(flag_const::CARRY)),
      branch_on_flag::ZERO => branch = !(branch ^ self.flag.get_at(flag_const::ZERO)),
      _ => return false,
    }
    if branch {
      // offset can be negative
      let offset = i8::from_le_bytes([self.read_and_forward_pc() as Byte]) as i32;
      self.skip_cycles += 1;
      let new_pc = (self.r_pc as i32 + offset) as Address;
      self.set_page_crossed(self.r_pc, new_pc, 2);
      self.r_pc = new_pc;
    } else {
      self.r_pc += 1;
    }
    true
  }

  fn execute_type1(&mut self, opcode: Byte) -> bool {
    if (opcode & INSTRUCTION_MODE_MASK) != 0x1 {
      return false;
    }
    // operation type
    let op = (opcode & OPERATION_MASK) >> OPERATION_SHIFT;
    // memory location
    let location = self.first_address_operation(
      (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT,
      op != operation1::STA,
    );
    if let None = location {
      return false;
    }
    // doing operation.
    if op == operation1::STA {
      self.main_bus.write(location.unwrap(), self.r_a);
    } else {
      let operand = self.main_bus.read(location.unwrap());
      match op {
        operation1::ORA => {
          self.r_a |= operand;
          self.set_zn(self.r_a);
        }
        operation1::AND => {
          self.r_a &= operand;
          self.set_zn(self.r_a);
        }
        operation1::EOR => {
          self.r_a ^= operand;
          self.set_zn(self.r_a);
        }
        operation1::ADC => {
          let r_a = self.r_a as Address;
          let operand = operand as Address;
          let sum = r_a + operand + (self.flag.get_at(flag_const::CARRY) as Address);
          // Carry forward or UNSIGNED overflow
          self.flag.set_at(flag_const::CARRY, bit_eq(sum, 0x100));
          // SIGNED overflow, would only happen if the sign of sum is
          // different from BOTH the operands
          self.flag.set_at(
            flag_const::OVERFLOW,
            bit_eq((r_a ^ sum) & (operand ^ sum), 0x80),
          );
          self.r_a = sum as Byte;
          self.set_zn(self.r_a);
        }
        operation1::LDA => {
          self.r_a = operand;
          self.set_zn(self.r_a);
        }
        operation1::SBC => {
          let r_a = self.r_a as Address;
          let operand = operand as Address;
          // High carry means "no borrow", thus negate and subtract
          let diff = (r_a)
            .wrapping_sub(operand)
            .wrapping_sub(!self.flag.get_at(flag_const::CARRY) as Address);
          // If the ninth bit is 1, the resulting number is negative =>
          // borrow => low carry
          self.flag.set_at(flag_const::CARRY, !bit_eq(diff, 0x100));
          // Same as ADC, except instead of the operand,
          // substitute with it's one complement
          self.flag.set_at(
            flag_const::OVERFLOW,
            bit_eq((r_a ^ diff) & (!operand ^ diff), 0x80),
          );
          self.r_a = diff as Byte;
          self.set_zn(self.r_a);
        }
        operation1::CMP => {
          self.compare(self.r_a, operand);
        }
        _ => return false,
      }
    }
    true
  }

  fn execute_type2(&mut self, opcode: Byte) -> bool {
    if opcode & INSTRUCTION_MODE_MASK != 0x2 {
      return false;
    }
    let op = (opcode & OPERATION_MASK) >> OPERATION_SHIFT;
    let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
    let location = match self
      .second_address_operation(addr_mode, op != operation2::LDX && op != operation2::STX)
    {
      None => return false,
      Some(v) => v,
    };
    match op {
      operation2::ASL => {
        let r = self.shift_left(addr_mode == addr_mode2::ACCUMULATOR, false, location);
        self.set_zn(r);
      }
      operation2::ROL => {
        let r = self.shift_left(addr_mode == addr_mode2::ACCUMULATOR, true, location);
        self.set_zn(r);
      }
      operation2::LSR => {
        let r = self.shift_right(addr_mode == addr_mode2::ACCUMULATOR, false, location);
        self.set_zn(r);
      }
      operation2::ROR => {
        let r = self.shift_right(addr_mode == addr_mode2::ACCUMULATOR, true, location);
        self.set_zn(r);
      }
      operation2::STX => self.main_bus.write(location, self.r_x),
      operation2::LDX => {
        self.r_x = self.main_bus.read(location);
        self.set_zn(self.r_x);
      }
      operation2::DEC => {
        let r = {
          let r = self.main_bus.read(location).wrapping_sub(1);
          self.main_bus.write(location, r);
          r
        };
        self.set_zn(r);
      }
      operation2::INC => {
        let r = {
          let r = self.main_bus.read(location).wrapping_add(1);
          self.main_bus.write(location, r);
          r
        };
        self.set_zn(r);
      }
      _ => return false,
    }
    true
  }

  fn execute_type0(&mut self, opcode: Byte) -> bool {
    if (opcode & INSTRUCTION_MODE_MASK) != 0x0 {
      return false;
    }
    let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
    if addr_mode == addr_mode2::ACCUMULATOR {
      return false;
    }
    let location = match self.second_address_operation(addr_mode, true) {
      None => return false,
      Some(v) => v,
    };

    match (opcode & OPERATION_MASK) >> OPERATION_SHIFT {
      operation0::BIT => {
        let operand = self.main_bus.read(location);
        self
          .flag
          .set_at(flag_const::ZERO, (self.r_a & operand) == 0);
        self.flag.set_by_check(flag_const::OVERFLOW, operand);
        self.flag.set_by_check(flag_const::NEGATIVE, operand);
      }
      operation0::STY => self.main_bus.write(location, self.r_y),
      operation0::LDY => {
        self.r_y = self.main_bus.read(location);
        self.set_zn(self.r_y);
      }
      operation0::CPY => {
        let val = self.main_bus.read(location);
        self.compare(self.r_y, val);
      }
      operation0::CPX => {
        let val = self.main_bus.read(location);
        self.compare(self.r_x, val);
      }
      _ => return false,
    }
    true
  }

  fn first_address_operation(
    &mut self,
    addr_mode: u8,
    check_page_crossed: bool,
  ) -> Option<Address> {
    let location = match addr_mode {
      addr_mode1::INDEXED_INDIRECT_X => {
        let zero_addr = self.r_x as Address + self.read_and_forward_pc();
        self.main_bus.read_addr(zero_addr & 0xFF)
          | self.main_bus.read_addr((zero_addr + 1) & 0xFF) << 8
      }
      addr_mode1::ZERO_PAGE => self.read_and_forward_pc(),
      addr_mode1::IMMEDIATE => {
        let old_pc = self.r_pc;
        self.r_pc += 1;
        old_pc
      }
      addr_mode1::ABSOLUTE => {
        let old_pc = self.r_pc;
        self.r_pc += 2;
        self.read_address(old_pc)
      }
      addr_mode1::INDIRECT_Y => {
        let zero_addr = self.read_and_forward_pc();
        let location = self.read_address(zero_addr);
        if check_page_crossed {
          self.set_page_crossed(location, location + self.r_y as Address, 1)
        }
        location + self.r_y as Address
      }
      // Address wraps around in the zero page
      addr_mode1::INDEXED_X => (self.read_and_forward_pc() + self.r_x as Address) & 0xFF,
      addr_mode1::ABSOLUTE_Y => self.read_addr_absolute(self.r_y as Address, check_page_crossed),
      addr_mode1::ABSOLUTE_X => self.read_addr_absolute(self.r_x as Address, check_page_crossed),
      _ => return None,
    };
    Some(location)
  }

  fn second_address_operation(&mut self, addr_mode: u8, index_x: bool) -> Option<Address> {
    let location = match addr_mode {
      addr_mode2::IMMEDIATE => {
        self.r_pc += 1;
        self.r_pc - 1
      }
      addr_mode2::ZERO_PAGE => self.read_and_forward_pc(),
      addr_mode2::ACCUMULATOR => 0,
      addr_mode2::ABSOLUTE => {
        self.r_pc += 2;
        self.read_address(self.r_pc - 2)
      }
      addr_mode2::INDEXED => {
        let index = if index_x { self.r_x } else { self.r_y } as Address;
        // The mask wraps address around zero page
        (self.read_and_forward_pc() + index) & 0xFF
      }
      addr_mode2::ABSOLUTE_INDEXED => {
        let location = self.read_address(self.r_pc);
        self.r_pc += 2;
        let index = if index_x { self.r_x } else { self.r_y } as Address;
        self.set_page_crossed(location, location + index, 1);
        location + index
      }
      _ => return None,
    };
    Some(location)
  }

  fn read_addr_absolute(&mut self, offset: Address, check_page_crossed: bool) -> Address {
    let location = self.read_address(self.r_pc);
    self.r_pc += 2;
    if check_page_crossed {
      self.set_page_crossed(location, location + offset, 1);
    }
    location + offset
  }

  fn shift_left(&mut self, accumulator: bool, rotate: bool, location: Address) -> Byte {
    let prev_c = self.flag.get_at(flag_const::CARRY) as Byte;
    let mut t;
    let operand = if accumulator {
      &mut self.r_a
    } else {
      t = self.main_bus.read(location);
      &mut t
    };

    self.flag.set_at(flag_const::CARRY, bit_eq(*operand, 0x80));
    *operand <<= 1;
    if rotate {
      // If Rotating,set the bit-0 to the previous carry
      *operand |= prev_c;
    }
    if !accumulator {
      self.main_bus.write(location, *operand);
    }
    *operand
  }

  fn shift_right(&mut self, accumulator: bool, rotate: bool, location: Address) -> Byte {
    let prev_c = self.flag.get_at(flag_const::CARRY) as Byte;
    let mut t;
    let operand = if accumulator {
      &mut self.r_a
    } else {
      t = self.main_bus.read(location);
      &mut t
    };
    self.flag.set_at(flag_const::CARRY, bit_eq(*operand, 1));
    *operand >>= 1;
    if rotate {
      // If Rotating, set the bit-7 to the previous carry
      *operand |= prev_c << 7;
    }
    if !accumulator {
      self.main_bus.write(location, *operand);
    }
    *operand
  }

  fn compare(&mut self, a: Byte, b: Byte) {
    let diff = a.overflowing_sub(b);
    self.flag.set_at(flag_const::CARRY, !diff.1);
    self.set_zn(diff.0 as Byte);
  }
}
