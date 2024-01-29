use rust_emu_common::types::*;

mod gb_opcodes;

use gb_opcodes::Instruction;

use self::gb_opcodes::{AddrMode, CondType, InstructionType, RegType};

mod flag_const {
  use rust_emu_common::types::*;
  // 7 6 5 4 3 2 1 0
  // Z N H C 0 0 0 0

  pub const ZERO: Byte = 1 << 7;
  pub const SUBTRACTION: Byte = 1 << 6;
  pub const HALF_CARRY: Byte = 1 << 5;
  pub const CARRY: Byte = 1 << 4;
  pub const ALL: Byte = 0xff;
}

pub struct GBCpu {
  skip_cycles: u32,
  cycles: u32,
  // registers
  r_pc: Address, // program counter
  r_sp: Address, // stack pointer
  r_a: Byte,
  r_f: Byte,
  // r_af: Address, // accumulator, high 8 bits is A, low 8 bits is cpu flags
  r_b: Byte,
  r_c: Byte,
  // r_bc: Address, // BC
  r_d: Byte,
  r_e: Byte,
  // r_de: Address, // DE
  r_h: Byte,
  r_l: Byte,
  // r_hl: Address, // HL
  cur_op: Address,
  cur_inst: Instruction,
  fetched_data: Address,
  mem_dis: Address,
  dist_in_mem: bool,

  int_master_enabled: bool,
}

impl GBCpu {
  pub fn step(&mut self) -> u32 {
    self.cycles += 1;
    if self.skip_cycles > 0 {
      return self.skip_cycles;
    }
    // handle interrupt

    let inst = self.fetch_instruction();
    let data = self.fetch_data(&inst);
    self.execute(inst);
    return self.skip_cycles;
  }

  fn fetch_instruction(&mut self) -> Instruction {
    let res = self.read_and_forward_pc();
    self.cur_op = res;
    gb_opcodes::decode_instruction(res)
  }

  fn fetch_data(&mut self, inst: &Instruction) -> Option<Address> {
    match inst.mode {
      AddrMode::IMP => None,
      AddrMode::R => Some(self.read_reg(inst.reg_1)),
      AddrMode::R2R => Some(self.read_reg(inst.reg_2)),
      AddrMode::R2D8 | AddrMode::R2A8 | AddrMode::D8 => {
        let res = self.read_and_forward_pc();
        self.skip_cycles += 1;
        Some(res)
      }
      AddrMode::R2D16 | AddrMode::D16 => {
        let low: u16 = self.read_and_forward_pc();
        let hight = self.read_and_forward_pc();
        let res = (hight as Address) << 8 | low as Address;
        self.skip_cycles += 2;
        Some(res)
      }
      AddrMode::MR2R => {
        let data = self.read_reg(inst.reg_2);
        self.mem_dis = self.read_reg(inst.reg_1);
        self.dist_in_mem = true;
        if inst.reg_1 == RegType::C {
          self.mem_dis |= 0xFF00;
        }
        Some(data)
      }
      AddrMode::R2MR => {
        let mut addr = self.read_reg(inst.reg_2);

        if inst.reg_2 == RegType::C {
          addr |= 0xFF00;
        }
        let data = self.read_bus(addr) as Address;
        self.skip_cycles += 1;
        Some(data)
      }
      AddrMode::R2HLI => {
        let data = self.read_bus(self.read_reg(inst.reg_2)) as Address;
        self.skip_cycles += 1;
        let reg_data = self.read_reg(RegType::HL) + 1;
        self.set_reg(RegType::HL, reg_data);
        Some(data)
      }
      AddrMode::R2HLD => {
        let data = self.read_bus(self.read_reg(inst.reg_2)) as Address;
        self.skip_cycles += 1;
        let reg_data = self.read_reg(RegType::HL) - 1;
        self.set_reg(RegType::HL, reg_data);
        Some(data)
      }
      AddrMode::HLI2R => {
        let data = self.read_reg(inst.reg_2);
        self.mem_dis = self.read_reg(inst.reg_1);
        self.dist_in_mem = true;
        let reg_data = self.read_reg(RegType::HL) + 1;
        self.set_reg(RegType::HL, reg_data);
        Some(data)
      }
      AddrMode::HLD2R => {
        let data = self.read_reg(inst.reg_2);
        self.mem_dis = self.read_reg(inst.reg_1);
        self.dist_in_mem = true;
        let reg_data = self.read_reg(RegType::HL) - 1;
        self.set_reg(RegType::HL, reg_data);
        Some(data)
      }
      AddrMode::HL2SPR => {
        // check this is right?
        let data = self.read_and_forward_pc();
        self.skip_cycles += 1;
        Some(data)
      }
      AddrMode::A82R => {
        self.mem_dis = self.read_and_forward_pc() | 0xFF00;
        self.dist_in_mem = true;
        self.skip_cycles += 1;
        None
      }
      // D162R
      AddrMode::A162R => {
        let low = self.read_and_forward_pc();
        let high = self.read_and_forward_pc();
        self.mem_dis = (high as Address) << 8 | low as Address;
        self.dist_in_mem = true;
        self.skip_cycles += 2;
        let data = self.read_reg(inst.reg_2);
        Some(data)
      }
      AddrMode::MR2D8 => {
        let data = self.read_and_forward_pc();
        self.skip_cycles += 1;
        self.mem_dis = self.read_reg(inst.reg_1);
        self.dist_in_mem = true;
        Some(data)
      }
      AddrMode::MR => {
        self.mem_dis = self.read_reg(inst.reg_1);
        self.dist_in_mem = true;
        let data = self.read_bus(self.read_reg(inst.reg_1)) as Address;
        self.skip_cycles += 1;
        Some(data)
      }
      AddrMode::R2A16 => {
        let low = self.read_and_forward_pc();
        let high = self.read_and_forward_pc();
        let addr = (high as Address) << 8 | low as Address;
        let res = self.read_bus(addr) as Address;
        self.skip_cycles += 3;
        Some(res)
      }
    }
  }

  fn execute(&mut self, inst: Instruction) {
    match inst.i_type {
      InstructionType::LD => self.inst_ld(inst),
      _ => {}
    }
  }

  fn inst_ld(&mut self, inst: Instruction) {
    if self.dist_in_mem {
      if inst.reg_2.is_16_bit() {
        // self.write_bus_addr(self.mem_dis, inst.param);
        self.write_bus(self.mem_dis, inst.param as Byte);
      } else {
        self.write_bus(self.mem_dis, inst.param as Byte);
      }

      self.skip_cycles += 1;
      return;
    }

    if inst.mode == AddrMode::HL2SPR {
      let hflag = ((self.read_reg(inst.reg_2) & 0xF) + (inst.param & 0xF)) >= 0x10;
      let cflag = ((self.read_reg(inst.reg_2) & 0xFF) + (inst.param & 0xFF)) >= 0x100;

      self.set_flags(None, None, Some(hflag), Some(cflag));
      self.set_reg(inst.reg_1, self.read_reg(inst.reg_2) + inst.param);
      return;
    }
    self.set_reg(inst.reg_1, inst.param);
  }

  fn inst_ldh(&mut self, inst: Instruction) {
    if inst.reg_1 == RegType::A {
      self.set_reg(inst.reg_1, self.read_bus(0xFF00 | inst.param) as Address);
    } else {
      self.write_bus(self.mem_dis, self.read_reg(RegType::A) as Byte)
    }
    self.skip_cycles += 1;
  }

  fn inst_jp(&mut self, inst: Instruction) {
    self.goto_addr(inst.cond, inst.param,   false);
  }

  fn inst_jr(&mut self, inst: Instruction) {
    let rel = inst.param & 0xFF;
    let addr = self.r_pc + rel as Address;
    self.goto_addr(inst.cond, addr, false);
  }

  fn inst_call(&mut self, inst: Instruction) {
    self.goto_addr(inst.cond, inst.param, true);
  }

  fn inst_rst(&mut self, inst: Instruction) {
    // TODO: difference between param and fetchdata?
    self.goto_addr(inst.cond, inst.param, true);
  }

  fn inst_ret(&mut self, inst: Instruction) {
    if inst.cond != CondType::NONE {
      self.skip_cycles += 1;
    }

    if check_cond(self.r_f, inst.cond) {
      let lo = self.pull_stack();
      let hi = self.pull_stack();
      self.r_pc = (hi as Address) << 8 | lo as Address;

      self.skip_cycles += 3;
    }
  }

  fn inst_reti(&mut self, inst: Instruction) {
    self.int_master_enabled = true;
    self.inst_ret(inst);
  }

  fn inst_pop(&mut self, inst: Instruction) {
    let n = self.pull_stack_16();
    if inst.reg_1 == RegType::AF {
      self.set_reg(inst.reg_1, n & 0xFFF0);
    } else {
      self.set_reg(inst.reg_1, n);
    }
  }

  fn inst_push(&mut self, inst: Instruction) {
    let hi = self.read_reg(inst.reg_1) as Byte >> 8 & 0xFF;
    self.push_stack(hi);
    let lo = self.read_reg(inst.reg_1) as Byte & 0xFF;
    self.push_stack(lo);
  }

  fn inst_inc(&mut self, inst: Instruction) {
    // if inst.reg_1.is_16_bit() {

    // }

    let val = if inst.reg_1 == RegType::HL && inst.mode == AddrMode::MR {
      let data = (self.read_reg(RegType::HL) + 1) as Byte;
      let target = self.read_reg(RegType::HL);
      self.write_bus(target, data);
      data
    } else {
      let data = self.read_reg(inst.reg_1) + 1;
      self.set_reg(inst.reg_1, data);
      self.read_reg(inst.reg_1) as Byte
    };
    if bit_eq(self.cur_op, 0x03) {
      return;
    } 
    self.set_flags(Some(val == 0), Some(false), Some(!bit_eq(val, 0x0F)),None)
  }

  fn goto_addr(&mut self, cond:CondType, addr: Address, push_pc: bool) {
    if check_cond(self.r_f, cond) {
      if push_pc {
        self.skip_cycles += 2;
        self.push_stack_16(self.r_pc);
      }

      self.r_pc = addr;
      self.skip_cycles += 1;
    }
  }

  fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
    let mut set_mask = 0;
    let mut unset_mask = 0xFF;
    if let Some(z) = z {
      if z {
        set_mask |= flag_const::ZERO;
      } else {
        unset_mask |= flag_const::ZERO;
      }
    }
    if let Some(n) = n {
      if n {
        set_mask |= flag_const::SUBTRACTION;
      } else {
        unset_mask |= flag_const::SUBTRACTION;
      }
    }
    if let Some(h) = h {
      if h {
        set_mask |= flag_const::HALF_CARRY;
      } else {
        unset_mask |= flag_const::HALF_CARRY;
      }
    }
    if let Some(c) = c {
      if c {
        set_mask |= flag_const::CARRY;
      } else {
        unset_mask |= flag_const::CARRY;
      }
    }
  }

  fn set_reg(&mut self, reg: RegType, data: Address) {}

  fn read_reg(&self, reg: RegType) -> Address {
    match reg {
      RegType::A => self.r_a.into(),
      RegType::F => self.r_f.into(),
      RegType::B => self.r_b.into(),
      RegType::C => self.r_c.into(),
      RegType::D => self.r_d.into(),
      RegType::E => self.r_e.into(),
      RegType::H => self.r_h.into(),
      RegType::L => self.r_l.into(),
      RegType::SP => self.r_sp,
      RegType::PC => self.r_pc,
      RegType::NONE => 0,
      RegType::AF => self.r_a as Address >> 8 | self.r_f as Address,
      RegType::BC => self.r_b as Address >> 8 | self.r_c as Address,
      RegType::DE => self.r_d as Address >> 8 | self.r_e as Address,
      RegType::HL => self.r_h as Address >> 8 | self.r_l as Address,
    }
  }

  fn read_and_forward_pc(&mut self) -> Address {
    // mock
    let res = self.read_bus(self.r_pc);
    self.r_pc += 1;
    res as Address
  }

  fn read_bus(&self, addr: Address) -> Byte {
    // moc
    0
  }

  fn write_bus(&mut self, addr: Address, value: Byte) {}


  fn push_stack(&mut self, value: Byte) {
    self.r_sp -= 1;
    self.write_bus(self.r_sp, value)
  }

  fn push_stack_16(&mut self, value: Address) {
    self.push_stack((value >> 8) as Byte);
    self.push_stack((value & 0xFF) as Byte);
  }

  fn pull_stack(&mut self) -> Byte {
    self.r_sp += 1;
    self.read_bus(self.r_sp)
  }

  fn pull_stack_16(&mut self) -> Address {
    let lo = self.pull_stack();
    let hi = self.pull_stack();

    (hi as Address) << 8 | lo as Address
  }
}

fn check_cond(f: Byte, cond: CondType) -> bool {
  match cond {
    CondType::NONE => true,
    CondType::NZ => !bit_eq(f, flag_const::ZERO),
    CondType::Z => bit_eq(f, flag_const::ZERO),
    CondType::NC => !bit_eq(f, flag_const::CARRY),
    CondType::C => bit_eq(f, flag_const::CARRY),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_cpu() {
    // let mut cpu = GBCpu {
    //   skip_cycles: 0,
    //   cycles: 0,
    //   r_pc: 0,
    //   r_sp: 0,
    //   r_a: 0,
    //   r_f: 0,
    //   r_b: 0,
    //   r_c: 0,
    //   r_d: 0,
    //   r_e: 0,
    //   r_h: 0,
    //   r_l: 0,
    //   cur_inst: Instruction::default(),
    //   fetched_data: 0,
    //   mem_dis: 0,
    //   dist_in_mem: false,
    // };
    // cpu.set_reg(RegType::A, 0x12);
    // cpu.set_reg(RegType::B, 0x34);
    // cpu.set_reg(RegType::C, 0x56);
    // cpu.set_reg(RegType::D, 0x78);
    // cpu.set_reg(RegType::E, 0x9A);
    // cpu.set_reg(RegType::H, 0xBC);
    // cpu.set_reg(RegType::L, 0xDE);
    // cpu.step();
  }
}
