use cpu::*;
use mem::Memory;
use snes::SNES;

#[derive(Debug)]
pub enum Value {
    Implied,
    Immediate8(u8),
    Immediate16(u16),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    IndirectX(u8),
    IndirectOffY(u8),
}

macro_rules! implied {
    ($instr:ident) => (Instruction(Opcode::$instr, Value::Implied))
}

macro_rules! absolute {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::Absolute($cpu.read_u16($mem))))
}

macro_rules! immediate_m {
    ($instr:ident, $cpu:ident, $mem:ident) => (match $cpu.p_reg.contains(FLAG_M) {
        true => Instruction(Opcode::$instr, Value::Immediate8($cpu.read_u8($mem))),
        false => Instruction(Opcode::$instr, Value::Immediate16($cpu.read_u16($mem))),
    })
}

macro_rules! immediate8 {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::Immediate8($cpu.read_u8($mem))))
}

#[derive(Debug)]
pub enum Opcode {
    CLC,
    SEI,
    STA,
    STZ,
    LDA,
    REP,
    SEP,
    XCE,
    Unknown(u8), 
}

pub struct Instruction(pub Opcode, pub Value);

impl Instruction {
    pub fn from(cpu: &mut Ricoh5A22, mem: &Memory) -> Instruction {
        match cpu.read_u8(mem) {
            0x18 => implied!(CLC),                                  // 0x18 CLC
            0x78 => implied!(SEI),                                  // 0x78 SEI
            0x8D => absolute!(STA, cpu, mem),                       // 0x8D STA addr
            0x9C => absolute!(STZ, cpu, mem),                       // 0x9C STZ addr
            0xA9 => immediate_m!(LDA, cpu, mem),                    // 0xA9 LDA #const
            0xC2 => immediate8!(REP, cpu, mem),                     // 0xC2 REP #const
            0xE2 => immediate8!(SEP, cpu, mem),                     // 0xE2 SEP #const
            0xFB => implied!(XCE),                                  // 0xFB XCE
            op => Instruction(Opcode::Unknown(op), Value::Implied),
        }
    }
}