use rust_emu_common::{component::main_bus::MainBus, types::*};
use serde::{Deserialize, Serialize};

mod gb_opcodes;

use gb_opcodes::Instruction;

use crate::{bus::GBAMainBus, cpu::flag_const::ZERO};


use self::{
  flag_const::{CARRY, HALF_CARRY, SUBTRACTION},
  gb_opcodes::{AddrMode, CondType, InstructionType, RegType},
};

use crate::interrupt::{*};

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

#[derive(Serialize, Deserialize)]
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
  int_flags: Byte,
  ie_register: Byte,
  enable_ime: bool,

  halt: bool,

  #[serde(skip)]
  main_bus: GBAMainBus,
}

impl GBCpu {
  pub fn new(main_bus: GBAMainBus) -> Self {
    Self {
        skip_cycles: 0,
        cycles: 0,
        r_pc: 0x100,
        r_sp: 0xFFFE,
        r_a: 0x01,
        r_f: 0xB0,
        r_b: 0x00,
        r_c: 0x13,
        r_d: 0x00,
        r_e: 0xD8,
        r_h: 0x01,
        r_l: 0x4D,
        cur_op: 0,
        cur_inst: Instruction::default(),
        fetched_data: 0,
        mem_dis: 0,
        dist_in_mem: false,
        int_master_enabled: false,
        int_flags: 0,
        ie_register: 0,
        enable_ime: false,
        halt: false,
        main_bus,
    }
  }

  pub fn set_main_bus(&mut self, main_bus: GBAMainBus) {
    self.main_bus = main_bus;
  }

  pub fn main_bus(&self) -> &GBAMainBus {
    &self.main_bus
  }

  pub fn main_bus_mut(&mut self) -> &mut GBAMainBus {
    &mut self.main_bus
  }

  pub fn reset(&mut self) {
    self.skip_cycles = 0;
    self.cycles = 0;
    self.r_pc = 0x100;
    self.r_sp = 0xFFFE;
    self.r_a = 0x01;
    self.r_f =0xBD;
    self.r_b = 0x00;
    self.r_c = 0x13;
    self.r_d = 0x00;
    self.r_e = 0xD8;
    self.r_h = 0x01;
    self.r_l = 0x4D;
    self.cur_op = 0;
    self.cur_inst = Instruction::default();
    self.fetched_data = 0;
    self.mem_dis = 0;
    self.dist_in_mem = false;
    self.int_master_enabled = false;
    self.int_flags = 0;
    self.ie_register = 0;
    self.enable_ime = false;
    self.halt = false;
  }
}

impl GBCpu {


  #[allow(dead_code)]
  pub fn step(&mut self) -> u8 {
    // self.cycles += 1;
    // if self.skip_cycles > 0 {
    //   return self.skip_cycles;
    // }
    // handle interrupt
    let cycle = if self.halt {
      if self.int_flags != 0 {
        self.halt = false;
      }
      self.cycles += 1;
      1
    } else {
      // let pc = self.r_pc;
      let mut inst = self.fetch_instruction();
      if let Some(data) = self.fetch_data(&inst) {
        inst.param = data;
        self.fetched_data = data;
      } else {
        // inst.param = self.fetched_data;
      }
      // debug print
      self.execute(&mut inst);
      self.cycles += inst.cycles as u32;
      // self.debug_print(pc, inst);
      inst.cycles
    };

    if self.int_master_enabled {
      self.handle_interrupts();
      self.enable_ime = false;
    }

    if self.enable_ime {
      self.int_master_enabled = true;
    }
    // return self.skip_cycles;
    cycle
  }

  #[allow(dead_code)]
  fn debug_flag(&self) -> String {
    format!("{}{}{}{}", 
      if bit_eq(self.r_f, ZERO) {'Z'} else {'-'},
      if bit_eq(self.r_f, SUBTRACTION) {'N'} else {'-'},
      if bit_eq(self.r_f, HALF_CARRY) {'H'} else {'-'},
      if bit_eq(self.r_f, CARRY) {'C'} else {'-'}, )
  }

  #[allow(dead_code)]
  fn debug_print(&mut self, pc: Address, inst: Instruction) {
    log::info!("{:08X} - {:04X}: ({:02X} {:04X} {:02X} {:02X}) A: {:02X} F: {} BC: {:02X}{:02X} DE: {:02X}{:02X} HL: {:02X}{:02X}",
      self.cycles * 4, pc, self.cur_op, inst.param,
      self.main_bus.read(pc+1), self.main_bus.read(pc+2),
      self.r_a, self.debug_flag(), self.r_b,self.r_c, self.r_d, self.r_e, self.r_h, self.r_l);
  }

  fn check_interrupt(&mut self, addr: Byte, int_flag: Byte) -> bool {
    if bit_eq(self.int_flags, int_flag) && bit_eq(self.ie_register, int_flag) {
      self.push_stack_16(self.r_pc);
      self.r_pc = addr as Address;

      self.int_flags &= !int_flag;
      self.halt = false;
      self.int_master_enabled = false;
      return true;
    }
    false
  }

  fn handle_interrupts(&mut self) {
    let _ = self.check_interrupt(0x40, INT_VBLANK)
      || self.check_interrupt(0x48, INT_LCD)
      || self.check_interrupt(0x50, INT_TIMER)
      || self.check_interrupt(0x58, INT_SERIAL)
      || self.check_interrupt(0x60, INT_JOYPAD);
  }

  pub(crate) fn trigger_interrupt(&mut self, interrupt: Byte) {
    self.int_flags |= interrupt;
  }

  fn fetch_instruction(&mut self) -> Instruction {
    let res = self.read_and_forward_pc();
    self.cur_op = res;
    gb_opcodes::decode_instruction(res)
  }

  fn fetch_data(&mut self, inst: &Instruction) -> Option<Address> {
    self.dist_in_mem = false;
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

  fn execute(&mut self, inst:&mut Instruction) {
    match inst.i_type {
      InstructionType::LD => self.inst_ld(*inst),
      InstructionType::LDH => self.inst_ldh(*inst),
      InstructionType::JP => self.inst_jp(inst),
      InstructionType::JR => self.inst_jr(inst),
      InstructionType::CALL => self.inst_call(inst),
      InstructionType::RST => self.inst_rst(*inst),
      InstructionType::RET => self.inst_ret(inst),
      InstructionType::RETI => self.inst_reti(inst),
      InstructionType::POP => self.inst_pop(*inst),
      InstructionType::PUSH => self.inst_push(*inst),
      InstructionType::INC => self.inst_inc(*inst),
      InstructionType::DEC => self.inst_dec(*inst),
      InstructionType::SUB => self.inst_sub(*inst),
      InstructionType::SBC => self.inst_sbc(*inst),
      InstructionType::ADC => self.inst_adc(*inst),
      InstructionType::ADD => self.inst_add(*inst),
      InstructionType::DAA => self.inst_daa(),
      InstructionType::CPL => self.inst_cpl(),
      InstructionType::SCF => self.inst_scf(),
      InstructionType::CCF => self.inst_ccf(),
      InstructionType::HALT => self.inst_halt(),
      InstructionType::RRA => self.inst_rra(),
      InstructionType::AND => self.inst_and(*inst),
      InstructionType::XOR => self.inst_xor(*inst),
      InstructionType::OR => self.inst_or(*inst),
      InstructionType::CP => self.inst_cp(*inst),
      InstructionType::DI => {self.int_master_enabled = false;},
      InstructionType::EI => {self.enable_ime = true;},
      InstructionType::STOP => self.inst_stop(),
      InstructionType::RLA => self.inst_rla(),
      InstructionType::RRCA => self.inst_rrca(),
      InstructionType::RLCA => self.inst_rlca(),
      InstructionType::CB => self.inst_cb(inst),
      InstructionType::NOP => {}
      InstructionType::RLC
      | InstructionType::RRC
      | InstructionType::RL
      | InstructionType::RR
      | InstructionType::SLA
      | InstructionType::SRA
      | InstructionType::SWAP
      | InstructionType::SRL
      | InstructionType::BIT
      | InstructionType::RES
      | InstructionType::SET => {
        log::error!("invalid instruction cb")
      }
      InstructionType::JPHL => {
        log::error!("no impl instruction error")
      }
      InstructionType::ERR => {
        log::error!("no impl instruction error")
      }
      InstructionType::NONE => {
        log::error!("invalid instruction")
      }
    }
  }

  fn inst_cb(&mut self, inst: &mut Instruction) {
    let op = inst.param;
    inst.cycles += 2;
    let reg = match op & 0x7 {
      0x0 => RegType::B,
      0x1 => RegType::C,
      0x2 => RegType::D,
      0x3 => RegType::E,
      0x4 => RegType::H,
      0x5 => RegType::L,
      0x6 => {
        inst.cycles += 2;
        RegType::HL
      },
      0x7 => RegType::A,
      _ => RegType::NONE,
    };
    let bit = (op >> 3) & 0x7;
    let bit_op = (op >> 6) & 0x3;
    let is_hl = reg == RegType::HL;
    let mut reg_val = if is_hl {
      self.read_bus(self.read_reg(reg))
    } else {
      self.read_reg(reg) as Byte
    };

    match bit_op {
      0x1 => {
        // BIT
        self.set_flags(
          Some(reg_val & (1 << bit) == 0),
          Some(false),
          Some(true),
          None,
        );
        return;
      }
      0x2 => {
        // RST
        reg_val &= !(1 << bit);
        self.set_reg8(reg, reg_val);
        return;
      }
      0x3 => {
        // SET
        reg_val |= 1 << bit;
        self.set_reg8(reg, reg_val);
        return;
      }
      _ => {}
    }

    let fc = self.check_flag(CARRY);

    match fc {
      0 => {
        // RLC
        let (result, c) = if bit_eq(reg_val, 0x80) {
          ((reg_val << 1) & 0xFF | 1, true)
        } else {
          (reg_val << 1 & 0xFF, false)
        };
        self.set_reg8(reg, result);
        self.set_flags(Some(result == 0), Some(false), Some(false), Some(c));
      }
      1 => {
        // RRC
        let old = reg_val;
        reg_val = reg_val >> 1 | old << 7;
        self.set_reg8(reg, reg_val);
        self.set_flags(
          Some(reg_val != 0),
          Some(false),
          Some(false),
          Some(bit_eq(old, 1)),
        )
      }
      2 => {
        // RL
        let old = reg_val;
        reg_val = reg_val << 1 | fc;
        self.set_reg8(reg, reg_val);
        self.set_flags(
          Some(reg_val != 0),
          Some(false),
          Some(false),
          Some(bit_eq(old, 0x80)),
        )
      }
      3 => {
        // RR
        let old = reg_val;
        reg_val = reg_val >> 1 | fc << 7;
        self.set_reg8(reg, reg_val);
        self.set_flags(
          Some(reg_val != 0),
          Some(false),
          Some(false),
          Some(bit_eq(old, 1)),
        )
      }
      4 => {
        // SLA
        let old = reg_val;
        reg_val = reg_val << 1;
        self.set_reg8(reg, reg_val);
        self.set_flags(
          Some(reg_val != 0),
          Some(false),
          Some(false),
          Some(bit_eq(old, 0x80)),
        )
      }
      5 => {
        // SRA
        let old = reg_val;
        reg_val = reg_val >> 1;
        self.set_reg8(reg, reg_val);
        self.set_flags(
          Some(reg_val != 0),
          Some(false),
          Some(false),
          Some(bit_eq(old, 1)),
        )
      }
      6 => {
        // SWAP
        reg_val = ((reg_val & 0xF0) >> 4) | ((reg_val & 0xF) << 4);
        self.set_reg8(reg, reg_val);
        self.set_flags(Some(reg_val == 0), Some(false), Some(false), Some(false))
      }
      7 => {
        // SRL
        let upd = reg_val >> 1;
        self.set_reg8(reg, upd);
        self.set_flags(
          Some(reg_val == 0),
          Some(false),
          Some(false),
          Some(bit_eq(reg_val, 1)),
        )
      }
      _ => {
        log::error!("invalid cb operation {:#02X}", bit_op);
      }
    }
  }

  fn inst_rlca(&mut self) {
    let u = self.r_a;
    let c = u >> 7;

    self.r_a = (u << 1) | c;
    self.set_flags(Some(false), Some(false), Some(false), Some(c == 1))
  }

  fn inst_rrca(&mut self) {
    let b = self.r_a & 1;
    self.r_a = (self.r_a >> 1) | (b << 7);
    self.set_flags(Some(false), Some(false), Some(false), Some(b == 1))
  }

  fn inst_rla(&mut self) {
    let u = self.r_a;
    let cf = self.check_flag(CARRY);
    let c = bit_eq(u >> 7, 1);

    self.r_a = (u << 1) | cf;
    self.set_flags(Some(false), Some(false), Some(false), Some(c))
  }

  fn inst_stop(&mut self) {
    log::error!("program calling stop");
  }

  fn inst_daa(&mut self) {
    let mut u = 0;
    let mut fc = false;
    let n_flag = bit_eq(self.r_f, SUBTRACTION);

    if bit_eq(self.r_f , HALF_CARRY) || (!n_flag && (self.r_a & 0xF) > 9) {
      u = 6;
    }
    if bit_eq(self.r_f, CARRY) || (!n_flag && self.r_a > 0x99) {
      u |= 0x60;
      fc = true;
    }

    if n_flag {
      self.r_a -= u;
    } else {
      self.r_a += u
    };
    self.set_flags(Some(self.r_a == 0), None, Some(false), Some(fc))
  }

  fn inst_cpl(&mut self) {
    self.r_a = !self.r_a;
    self.set_flags(None, Some(true), Some(true), None);
  }

  fn inst_scf(&mut self) {
    self.set_flags(None, Some(false), Some(false), Some(true));
  }

  fn inst_ccf(&mut self) {
    self.set_flags(
      None,
      Some(false),
      Some(false),
      Some(self.check_flag(CARRY) == 0),
    );
  }

  fn inst_halt(&mut self) {
    self.halt = true;
  }

  fn inst_rra(&mut self) {
    let carry = self.check_flag(CARRY);
    let n_c = bit_eq(self.r_a, 1);

    self.r_a = (self.r_a >> 1) | (carry << 7);
    self.set_flags(Some(self.r_a == 0), Some(false), Some(false), Some(n_c))
  }

  fn inst_and(&mut self, inst: Instruction) {
    self.r_a &= inst.param as Byte & 0xFF;
    self.set_flags(Some(self.r_a == 0), Some(false), Some(true), Some(false));
  }

  fn inst_xor(&mut self, inst: Instruction) {
    self.r_a ^= inst.param as Byte & 0xFF;
    self.set_flags(Some(self.r_a == 0), Some(false), Some(false), Some(false));
  }

  fn inst_or(&mut self, inst: Instruction) {
    self.r_a |= inst.param as Byte & 0xFF;
    self.set_flags(Some(self.r_a == 0), Some(false), Some(false), Some(false));
  }

  fn inst_cp(&mut self, inst: Instruction) {
    let ret = (self.r_a as Address).overflowing_sub(inst.param);
    let h = (self.r_a as Address) & 0xF < inst.param & 0xF;
    self.set_flags(Some(ret.0 == 0), Some(true), Some(h), Some(ret.1))

    // let ret = (self.r_a as i16) - (inst.param as i16);
    // let h = (self.r_a as i16) & 0xF < (inst.param as i16) & 0xF;
    // self.set_flags(Some(ret == 0), Some(true), Some(h), Some(ret < 0))
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
      self.set_reg(inst.reg_1, (self.read_reg(inst.reg_2) as i16 + inst.param as i16) as Address);
      return;
    }
    self.set_reg(inst.reg_1, inst.param);
  }

  fn inst_ldh(&mut self, inst: Instruction) {
    if inst.reg_1 == RegType::A {
      // log::info!("ldh read bus {:X} {:X}", 0xFF00 | inst.param, self.read_bus(0xFF00 | inst.param));
      let data = self.read_bus(0xFF00 | inst.param) as Address;
      self.set_reg(inst.reg_1, data);
    } else {
      self.write_bus(self.mem_dis, self.read_reg(RegType::A) as Byte)
    }
    // self.skip_cycles += 1;
  }

  fn inst_jp(&mut self, inst:&mut Instruction) {
    if self.goto_addr(inst.cond, inst.param, false) && inst.cond != CondType::NONE {
      inst.cycles += 1;
    }
  }

  fn inst_jr(&mut self, inst:&mut Instruction) {
    let rel = ((inst.param & 0xFF) as i8) as i16;
    let addr = self.r_pc as i16 + rel;
    // log::info!("jr {} {} {}", addr ,rel, self.r_pc);
    if self.goto_addr(inst.cond, addr as u16, false) && inst.cond != CondType::NONE {
      inst.cycles += 1;
    }
  }

  fn inst_call(&mut self, inst:& mut Instruction) {
    if self.goto_addr(inst.cond, inst.param, true) {
      inst.cycles += 3;
    }
  }

  fn inst_rst(&mut self, inst: Instruction) {
    // TODO: difference between param and fetchdata?
    self.goto_addr(inst.cond, inst.param, true);
  }

  fn inst_ret(&mut self, inst:&mut Instruction) {
    // if inst.cond != CondType::NONE {
    //   self.skip_cycles += 1;
    // }

    if check_cond(self.r_f, inst.cond) {
      let lo = self.pull_stack();
      let hi = self.pull_stack();
      self.r_pc = (hi as Address) << 8 | lo as Address;

      // self.skip_cycles += 3;
      if inst.cond != CondType::NONE {
        inst.cycles += 3;
      }
    }
  }

  fn inst_reti(&mut self, inst:&mut Instruction) {
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
    let hi = (self.read_reg(inst.reg_1) >> 8) as Byte & 0xFF;
    self.push_stack(hi);
    let lo = self.read_reg(inst.reg_1) as Byte & 0xFF;
    self.push_stack(lo);
  }

  fn inst_inc(&mut self, inst: Instruction) {
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
    self.set_flags(Some(val == 0), Some(false), Some((val & 0x0F) == 0), None)
  }

  fn inst_dec(&mut self, inst: Instruction) {
    let val = if inst.reg_1 == RegType::HL && inst.mode == AddrMode::MR {
      let data = (self.read_reg(RegType::HL) - 1) as Byte;
      let target = self.read_reg(RegType::HL);
      self.write_bus(target, data);
      data
    } else {
      let data = self.read_reg(inst.reg_1).overflowing_sub(1).0;
      self.set_reg(inst.reg_1, data);
      self.read_reg(inst.reg_1) as Byte
    };
    if bit_eq(self.cur_op, 0x0B) {
      return;
    }
    self.set_flags(Some(val == 0), Some(true), Some(bit_eq(val, 0x0F)), None)
  }

  fn inst_sub(&mut self, inst: Instruction) {
    let reg_value = self.read_reg(inst.reg_1);
    let val = reg_value - inst.param;
    let h = reg_value & 0xF < inst.param & 0xF;
    let c = reg_value < inst.param;

    self.set_reg(inst.reg_1, val);
    self.set_flags(Some(val == 0), Some(true), Some(h), Some(c))
  }

  fn inst_sbc(&mut self, inst: Instruction) {
    let flag_c = if bit_eq(self.r_f, CARRY) { 1 } else { 0 };
    let value = inst.param + flag_c;
    let reg = self.read_reg(inst.reg_1);
    let caled_value = reg - value;
    let h = reg & 0xF < value & 0xF;
    let c = reg < value;
    self.set_reg(inst.reg_1, caled_value);
    self.set_flags(Some(caled_value == 0), Some(true), Some(h), Some(c))
  }

  fn inst_adc(&mut self, inst: Instruction) {
    let u = inst.param;
    let a = self.r_a;
    let c = if bit_eq(self.r_f, CARRY) { 1 } else { 0 };
    let add_res = (a as Address + u + c) as Address;
    self.r_a = add_res as Byte;

    self.set_flags(
      Some(self.r_a == 0),
      Some(false),
      Some((a as Address & 0xF) + (u & 0xF) + c > 0xF),
      Some(add_res > 0xFF),
    )
  }

  fn inst_add(&mut self, inst: Instruction) {
    let reg_val = self.read_reg(inst.reg_1);
    let val = reg_val.overflowing_add(inst.param);

    let (z, h, c) = if !inst.reg_1.is_16_bit() || inst.reg_1 == RegType::SP {
      let z = if inst.reg_1 == RegType::SP {
        false
      } else {
        val.0 & 0xFF == 0
      };
      let h = reg_val & 0xF + inst.param & 0xF > 0xF;
      let c = reg_val & 0xFF + inst.param & 0xFF > 0xFF;
      (Some(z), Some(h), Some(c))
    } else {
      let h = reg_val & 0xFFF + inst.param & 0xFFF > 0xFFF;
      let c = val.1;
      (None, Some(h), Some(c))
    };
    self.set_reg(inst.reg_1, val.0);
    self.set_flags(z, Some(false), h, c)
  }

  fn goto_addr(&mut self, cond: CondType, addr: Address, push_pc: bool) -> bool {
    if check_cond(self.r_f, cond) {
      if push_pc {
        self.skip_cycles += 2;
        self.push_stack_16(self.r_pc);
      }

      self.r_pc = addr;
      self.skip_cycles += 1;
      // log::info!("jump {:04X}", addr);
      true
    } else {
      // log::info!("not jump");
      false
    }
  }

  #[inline]
  fn check_flag(&self, flag: Byte) -> Byte {
    if self.r_f & flag != 0 {
      1
    } else {
      0
    }
  }

  fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
    let mut set_mask = 0;
    let mut unset_mask = 0;
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
    self.r_f = (self.r_f & !unset_mask) | set_mask;
  }

  fn set_reg8(&mut self, reg: RegType, data: Byte) {
    if reg == RegType::HL {
      self.write_bus(self.read_reg(RegType::HL), data)
    } else {
      self.set_reg(reg, data as Address)
    }
  }

  fn set_reg(&mut self, reg: RegType, data: Address) {
    match reg {
      RegType::A => self.r_a = (data & 0xFF) as Byte,
      RegType::F => self.r_f = (data & 0xFF) as Byte,
      RegType::B => self.r_b = (data & 0xFF) as Byte,
      RegType::C => self.r_c = (data & 0xFF) as Byte,
      RegType::D => self.r_d = (data & 0xFF) as Byte,
      RegType::E => self.r_e = (data & 0xFF) as Byte,
      RegType::H => self.r_h = (data & 0xFF) as Byte,
      RegType::L => self.r_l = (data & 0xFF) as Byte,
      RegType::SP => self.r_sp = data,
      RegType::PC => self.r_pc = data,
      RegType::AF => {
        self.r_a = ((data & 0xFF00) >> 8) as Byte;
        self.r_f = (data & 0xFF) as Byte;
      }
      RegType::BC => {
        self.r_b = ((data & 0xFF00) >> 8) as Byte;
        self.r_c = (data & 0xFF) as Byte;
      }
      RegType::DE => {
        self.r_d = ((data & 0xFF00) >> 8) as Byte;
        self.r_e = (data & 0xFF) as Byte;
      }
      RegType::HL => {
        self.r_h = ((data & 0xFF00) >> 8) as Byte;
        self.r_l = (data & 0xFF) as Byte;
      }
      RegType::NONE => (),
    }
  }

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
      RegType::AF => (self.r_a as Address) << 8 | self.r_f as Address,
      RegType::BC => (self.r_b as Address) << 8 | self.r_c as Address,
      RegType::DE => (self.r_d as Address) << 8 | self.r_e as Address,
      RegType::HL => (self.r_h as Address) << 8 | self.r_l as Address,
      RegType::NONE => 0,
    }
  }

  #[inline]
  fn read_and_forward_pc(&mut self) -> Address {
    let res = self.read_bus(self.r_pc);
    self.r_pc += 1;
    res as Address
  }

  #[inline]
  fn read_bus(&mut self, addr: Address) -> Byte {
    if addr == 0xFFFF {
      return self.ie_register;
    }
    self.main_bus.read(addr)
  }

  #[inline]
  fn write_bus(&mut self, addr: Address, value: Byte) {
    if addr == 0xFFFF {
      self.ie_register = value;
      return;
    }
    self.main_bus.write(addr, value)
  }

  #[inline]
  fn push_stack(&mut self, value: Byte) {
    // self.r_sp -= 1;
    self.r_sp = self.r_sp.overflowing_sub(1).0;
    self.write_bus(self.r_sp, value)
  }

  #[inline]
  fn push_stack_16(&mut self, value: Address) {
    self.push_stack((value >> 8) as Byte);
    self.push_stack((value & 0xFF) as Byte);
  }

  #[inline]
  fn pull_stack(&mut self) -> Byte {
    let ret = self.read_bus(self.r_sp);
    self.r_sp += 1;
    ret
  }

  #[inline]
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
