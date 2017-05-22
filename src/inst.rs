use cpu::Ricoh5A22;
use mem::Memory;
use snes::SNES;

#[derive(Debug)]
pub enum Value {
    Implied,
    Immediate(u8),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    IndirectX(u8),
    IndirectOffY(u8),
}

#[derive(Debug)]
pub enum Opcode {
    CLC,        // 0x18
    SEI,        // 0x78
    STZ,        // 0x9C
    XCE,        // 0xFB
    Unknown(u8), 
}

pub struct Instruction(pub Opcode, pub Value);

macro_rules! implied {
    ($instr:ident) => (Instruction(Opcode::$instr, Value::Implied))
}

macro_rules! absolute {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::Absolute($cpu.read_u16($mem))))
}

impl Instruction {
    pub fn from(cpu: &mut Ricoh5A22, mem: &Memory) -> Instruction {
        match cpu.read_u8(mem) {
            0x18 => implied!(CLC),
            0x78 => implied!(SEI),
            0x9C => absolute!(STZ, cpu, mem),
            0xFB => implied!(XCE),
            op => Instruction(Opcode::Unknown(op), Value::Implied),
        }
    }
}