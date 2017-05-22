use cpu::Ricoh5A22;
use mem::Memory;
use snes::SNES;

pub enum Value {
    Implied,
}

pub enum Opcode {
    Unknown(u8),
}

pub struct Instruction(pub Opcode, pub Value);

impl Instruction {
    pub fn from(cpu: &mut Ricoh5A22, mem: &Memory) -> Instruction {
        match cpu.read_u8(mem) {
            op => Instruction(Opcode::Unknown(op), Value::Implied)
        }
    }
}
