use inst::{Instruction, Opcode, Value};
use cart::SnesCart;
use mem::Memory;
use snes::SNES;

#[derive(Debug, Clone)]
pub struct Ricoh5A22 {
    pc: u16
}

impl Ricoh5A22 {
    pub fn new() -> Ricoh5A22 {
        Ricoh5A22 {
            pc: 0,
        }
    }

    pub fn reset(&mut self, cart: &SnesCart) {
        self.pc = (cart[0x7FFC] as u16) | ((cart[0x7FFD] as u16) << 8);
        println!("CPU Reset, PC: {:#X}", self.pc);
    }

    pub fn step(&mut self, mem: &Memory) -> u64 {
        match Instruction::from(self, mem) {
            Instruction(Opcode::Unknown(op), _) => {
                panic!("Unknown instruction: {:X}", op);
            }
            _ => unimplemented!()
        }
        1
    }

    pub fn read_u8(&mut self, mem: &Memory) -> u8 {
        let val = mem.peek_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }
}