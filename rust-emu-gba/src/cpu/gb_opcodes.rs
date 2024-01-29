use rust_emu_common::types::Address;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum AddrMode {
  IMP,
  R2D16,
  R2R,
  MR2R,
  R,
  R2D8,
  R2MR,
  R2HLI,
  R2HLD,
  HLI2R,
  HLD2R,
  R2A8,
  A82R,
  HL2SPR,
  D16,
  D8,
  // D162R,
  MR2D8,
  MR,
  A162R,
  R2A16,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub(crate) enum RegType {
  NONE,
  A,
  F,
  B,
  C,
  D,
  E,
  H,
  L,
  AF,
  BC,
  DE,
  HL,
  SP,
  PC,
}

impl RegType {
  pub fn is_16_bit(self) -> bool {
    self >= RegType::AF
  }
}

#[derive(Clone, Copy)]
pub(crate) enum InstructionType {
  NONE,
  NOP,
  LD,
  INC,
  DEC,
  RLCA,
  ADD,
  RRCA,
  STOP,
  RLA,
  JR,
  RRA,
  DAA,
  CPL,
  SCF,
  CCF,
  HALT,
  ADC,
  SUB,
  SBC,
  AND,
  XOR,
  OR,
  CP,
  POP,
  JP,
  PUSH,
  RET,
  CB,
  CALL,
  RETI,
  LDH,
  JPHL,
  DI,
  EI,
  RST,
  ERR,
  // CB instructions...
  RLC,
  RRC,
  RL,
  RR,
  SLA,
  SRA,
  SWAP,
  SRL,
  BIT,
  RES,
  SET,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CondType {
  NONE,
  NZ,
  Z,
  NC,
  C,
}

#[derive(Clone, Copy)]
pub(crate) struct Instruction {
  pub(crate) i_type: InstructionType,
  pub(crate) mode: AddrMode,
  pub(crate) reg_1: RegType,
  pub(crate) reg_2: RegType,
  pub(crate) cond: CondType,
  // pub(crate) param: Byte,
  pub(crate) param: Address,
}

impl Default for Instruction {
  fn default() -> Self {
    Instruction {
      i_type: InstructionType::NOP,
      mode: AddrMode::IMP,
      reg_1: RegType::NONE,
      reg_2: RegType::NONE,
      cond: CondType::NONE,
      param: 0
    }
  }
}

#[macro_export]
macro_rules! create_struct {
    ($struct_name:ident, $($field:ident = $value:expr),*) => {
        {
            let mut instance = $struct_name::default();
            $(instance.$field = $value;)*
            instance
        }
    };
}

macro_rules! create_instruction {
    ($($field:ident = $value:expr),*) => {
      // WTF?
        // create_struct!(Instruction, $($field = $value),*)
        {
          let mut instance = Instruction{
            i_type: InstructionType::NOP,
            mode: AddrMode::IMP,
            reg_1: RegType::NONE,
            reg_2: RegType::NONE,
            cond: CondType::NONE,
            param: 0
          };
          $(instance.$field = $value;)*
          instance
        }
    };
}

#[rustfmt::skip]
static INSTRUCTIONS: [Instruction;0x100] = [
// 0x0X
create_instruction!(i_type = InstructionType::NOP),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D16, reg_1 =  RegType::BC),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 =  RegType::BC, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 =  RegType::BC ),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 =  RegType::B),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 =  RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::B),
create_instruction!(i_type = InstructionType::RLCA),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::A162R, reg_2 = RegType::SP),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::HL, reg_2 = RegType::BC),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::BC),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::BC),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::C),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::C),
create_instruction!(i_type = InstructionType::RRCA),

//0x1X
create_instruction!(i_type = InstructionType::STOP),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D16, reg_1 = RegType::DE),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::DE, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::DE),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::D),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::D),
create_instruction!(i_type = InstructionType::RLA),
create_instruction!(i_type = InstructionType::JR, mode = AddrMode::D8),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::HL, reg_2 = RegType::DE),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::DE),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::DE),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::E),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::E),
create_instruction!(i_type = InstructionType::RRA),


//0x2X
create_instruction!(i_type = InstructionType::JR, mode = AddrMode::D8, cond = CondType::NZ),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D16, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::HLI2R, reg_1 = RegType::HL, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::H),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::H),
create_instruction!(i_type = InstructionType::DAA),
create_instruction!(i_type = InstructionType::JR, mode = AddrMode::D8, cond = CondType::Z),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::HL, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2HLI, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::L),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::L),
create_instruction!(i_type = InstructionType::CPL),

//0x3X
create_instruction!(i_type = InstructionType::JR, mode = AddrMode::D8, cond = CondType::NC),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D16, reg_1 = RegType::SP),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::HLD2R, reg_1 = RegType::HL, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::SP),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::MR, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::MR, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2D8, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::SCF),
create_instruction!(i_type = InstructionType::JR, mode = AddrMode::D8, cond = CondType::C),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::HL, reg_2 = RegType::SP),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2HLD, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::SP),
create_instruction!(i_type = InstructionType::INC, mode = AddrMode::R, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::DEC, mode = AddrMode::R, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::CCF),

//0x4X
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::B, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::B, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::C, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::C, reg_2 = RegType::A),

//0x5X
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::D, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::D, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::E, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::E, reg_2 = RegType::A),

//0x6X
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::H, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::H, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::L, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::L, reg_2 = RegType::A),

//0x7X
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::HALT),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_1 = RegType::HL, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),

//0x8X
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),

//0x9X
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),

//0xAX
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),

//0xBX
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::B),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::D),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::E),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::H),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::L),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2R, reg_1 = RegType::A, reg_2 = RegType::A),

// 0xCX
create_instruction!(i_type = InstructionType::RET, mode = AddrMode::IMP, cond = CondType::NZ),
create_instruction!(i_type = InstructionType::POP, mode = AddrMode::R2R, reg_1 = RegType::BC),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::D16, cond = CondType::NZ),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::D16),
create_instruction!(i_type = InstructionType::CALL, mode = AddrMode::D16, cond = CondType::NZ),
create_instruction!(i_type = InstructionType::PUSH, mode = AddrMode::R, reg_1 = RegType::BC),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x00),
create_instruction!(i_type = InstructionType::RET, mode = AddrMode::IMP, cond = CondType::Z),
create_instruction!(i_type = InstructionType::RET),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::D16, cond = CondType::Z),
create_instruction!(i_type = InstructionType::CB, mode = AddrMode::D8),
create_instruction!(i_type = InstructionType::CALL, mode = AddrMode::D16, cond = CondType::Z),
create_instruction!(i_type = InstructionType::CALL, mode = AddrMode::D16),
create_instruction!(i_type = InstructionType::ADC, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x08),

// 0xDX
create_instruction!(i_type = InstructionType::RET, mode = AddrMode::IMP, cond = CondType::NC),
create_instruction!(i_type = InstructionType::POP, mode = AddrMode::R, reg_1 = RegType::DE),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::D16, cond = CondType::NC),
create_instruction!(i_type = InstructionType::NONE), // 0xD3
create_instruction!(i_type = InstructionType::CALL, mode = AddrMode::D16, cond = CondType::NC),
create_instruction!(i_type = InstructionType::PUSH, mode = AddrMode::R, reg_1 = RegType::DE),
create_instruction!(i_type = InstructionType::SUB, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x10),
create_instruction!(i_type = InstructionType::RET, mode = AddrMode::IMP, cond = CondType::C),
create_instruction!(i_type = InstructionType::RETI),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::D16, cond = CondType::C),
create_instruction!(i_type = InstructionType::NONE), // 0xDB
create_instruction!(i_type = InstructionType::CALL, mode = AddrMode::D16, cond = CondType::C),
create_instruction!(i_type = InstructionType::NONE), // 0xDD
create_instruction!(i_type = InstructionType::SBC, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x18),

//0xEX
create_instruction!(i_type = InstructionType::LDH, mode = AddrMode::A82R, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::POP, mode = AddrMode::R, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::MR2R, reg_2 = RegType::C, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::NONE), // 0xE3
create_instruction!(i_type = InstructionType::NONE), // 0xE4
create_instruction!(i_type = InstructionType::PUSH, mode = AddrMode::R, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::AND, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x20),
create_instruction!(i_type = InstructionType::ADD, mode = AddrMode::R2D8, reg_1 = RegType::SP),
create_instruction!(i_type = InstructionType::JP, mode = AddrMode::R, reg_1 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::A162R, reg_2 = RegType::A),
create_instruction!(i_type = InstructionType::NONE), // 0xEB
create_instruction!(i_type = InstructionType::NONE), // 0xEC
create_instruction!(i_type = InstructionType::NONE), // 0xED
create_instruction!(i_type = InstructionType::XOR, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x28),

//0xFX
create_instruction!(i_type = InstructionType::LDH, mode = AddrMode::R2A8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::POP, mode = AddrMode::R, reg_1 = RegType::AF),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2MR, reg_1 = RegType::A, reg_2 = RegType::C),
create_instruction!(i_type = InstructionType::DI),
create_instruction!(i_type = InstructionType::NONE), // 0xF4
create_instruction!(i_type = InstructionType::PUSH, mode = AddrMode::R, reg_1 = RegType::AF),
create_instruction!(i_type = InstructionType::OR, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x30),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::HL2SPR, reg_1 = RegType::HL, reg_2 = RegType::SP),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2R, reg_1 = RegType::SP, reg_2 = RegType::HL),
create_instruction!(i_type = InstructionType::LD, mode = AddrMode::R2A16, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::EI),
create_instruction!(i_type = InstructionType::NONE), // 0xFC
create_instruction!(i_type = InstructionType::NONE), // 0xFD
create_instruction!(i_type = InstructionType::CP, mode = AddrMode::R2D8, reg_1 = RegType::A),
create_instruction!(i_type = InstructionType::RST, mode = AddrMode::IMP, param = 0x38),
];

impl std::fmt::Display for InstructionType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      InstructionType::NONE => "<NONE>",
      InstructionType::NOP => "NOP",
      InstructionType::LD => "LD",
      InstructionType::INC => "INC",
      InstructionType::DEC => "DEC",
      InstructionType::RLCA => "RLCA",
      InstructionType::ADD => "ADD",
      InstructionType::RRCA => "RRCA",
      InstructionType::STOP => "STOP",
      InstructionType::RLA => "RLA",
      InstructionType::JR => "JR",
      InstructionType::RRA => "RRA",
      InstructionType::DAA => "DAA",
      InstructionType::CPL => "CPL",
      InstructionType::SCF => "SCF",
      InstructionType::CCF => "CCF",
      InstructionType::HALT => "HALT",
      InstructionType::ADC => "ADC",
      InstructionType::SUB => "SUB",
      InstructionType::SBC => "SBC",
      InstructionType::AND => "AND",
      InstructionType::XOR => "XOR",
      InstructionType::OR => "OR",
      InstructionType::CP => "CP",
      InstructionType::POP => "POP",
      InstructionType::JP => "JP",
      InstructionType::PUSH => "PUSH",
      InstructionType::RET => "RET",
      InstructionType::CB => "CB",
      InstructionType::CALL => "CALL",
      InstructionType::RETI => "RETI",
      InstructionType::LDH => "LDH",
      InstructionType::JPHL => "JPHL",
      InstructionType::DI => "DI",
      InstructionType::EI => "EI",
      InstructionType::RST => "RST",
      InstructionType::ERR => "IN_ERR",
      InstructionType::RLC => "IN_RLC",
      InstructionType::RRC => "IN_RRC",
      InstructionType::RL => "IN_RL",
      InstructionType::RR => "IN_RR",
      InstructionType::SLA => "IN_SLA",
      InstructionType::SRA => "IN_SRA",
      InstructionType::SWAP => "IN_SWAP",
      InstructionType::SRL => "IN_SRL",
      InstructionType::BIT => "IN_BIT",
      InstructionType::RES => "IN_RES",
      InstructionType::SET => "IN_SET",
    };
    write!(f, "{}", s)
  }
}

impl std::fmt::Display for RegType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      RegType::NONE => "<NONE>",
      RegType::A => "A",
      RegType::B => "B",
      RegType::C => "C",
      RegType::D => "D",
      RegType::E => "E",
      RegType::H => "H",
      RegType::L => "L",
      RegType::F => "F",
      RegType::BC => "BC",
      RegType::DE => "DE",
      RegType::HL => "HL",
      RegType::SP => "SP",
      RegType::AF => "AF",
      RegType::PC => "PC",
    };
    write!(f, "{}", s)
  }
}

impl std::fmt::Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ", self.i_type)?;
    match self.mode {
      AddrMode::IMP => Ok(()),
      AddrMode::R2D16 | AddrMode::R2A16 => write!(f, "{},${:04X}", self.reg_1, self.param),
      AddrMode::R => write!(f, "{}", self.reg_1),
      AddrMode::R2R => write!(f, "{},{}", self.reg_1, self.reg_2),
      AddrMode::MR2R => write!(f, "({}),{}", self.reg_1, self.reg_2),
      AddrMode::MR => write!(f, "({})", self.reg_1),
      AddrMode::R2MR => write!(f, "{},({})", self.reg_1, self.reg_2),
      AddrMode::R2D8 | AddrMode::R2A8 => write!(f, "{},${:02X}", self.reg_1, self.param & 0xFF),
      AddrMode::R2HLI => write!(f, "{},({}+)", self.reg_1, self.reg_2),
      AddrMode::R2HLD => write!(f, "{},({}-)", self.reg_1, self.reg_2),
      AddrMode::HLI2R => write!(f, "({}+),{}", self.reg_1, self.reg_2),
      AddrMode::HLD2R => write!(f, "({}-),{}", self.reg_1, self.reg_2),
      AddrMode::A82R => write!(f, "(PC-1),{}", self.reg_2),
      AddrMode::HL2SPR => write!(f, "({}+),SP+{}", self.reg_1, self.param & 0xFF),
      AddrMode::D8 => write!(f, "${:02X}", self.param & 0xFF),
      AddrMode::D16 => write!(f, "${:04X}", self.param),
      AddrMode::MR2D8 => write!(f, "({}),{:02X}", self.reg_1, self.param),
      AddrMode::A162R => write!(f, "({:04x}),{}", self.param, self.reg_2),
      // AddrMode::D162R => Ok(()), // Invalid instruction?
    }
  }
}

pub(crate) fn decode_instruction(opcode: Address) -> Instruction {
  INSTRUCTIONS[opcode as usize]
}
