use inst::{Instruction, Opcode, Value};
use cart::SnesCart;
use mem::Memory;
use snes::SNES;

use bitflags;

bitflags! {
    flags PReg: u8 {
        const FLAG_C = 0b00000001,
        const FLAG_Z = 0b00000010,
        const FLAG_I = 0b00000100,
        const FLAG_D = 0b00001000,
        const FLAG_X = 0b00010000,
        const FLAG_M = 0b00100000,
        const FLAG_V = 0b01000000,
        const FLAG_N = 0b10000000,
    }
}

#[derive(Debug, Clone)]
pub struct Ricoh5A22 {
    pc: u16,
    p_reg: PReg,
    nmitimen: u8,
    emulation: bool,
}

impl Ricoh5A22 {
    pub fn new() -> Ricoh5A22 {
        Ricoh5A22 {
            pc:        0u16,
            p_reg:     PReg::empty(),
            nmitimen:  0u8,
            emulation: true,
        }
    }

    pub fn reset(&mut self, cart: &SnesCart) {
        self.pc = (cart[0x7FFC] as u16) | ((cart[0x7FFD] as u16) << 8);
        println!("CPU Reset, PC: ${:X}", self.pc);
    }

    pub fn step(&mut self, mem: &Memory) -> u64 {
        print!("0x{:4X}: ", self.pc);
        match Instruction::from(self, mem) {
            Instruction(Opcode::SEI, _) => {
                println!("SEI");
                self.p_reg.insert(FLAG_I);
                2
            }
            Instruction(Opcode::STZ, Value::Absolute(addr)) => {
                println!("STZ ${:X}", addr);

                match self.p_reg.contains(FLAG_M) {
                    true => {
                        mem.write_u8(addr, 0u8);
                        4
                    }
                    false => {
                        mem.write_u8(addr, 0u8);
                        mem.write_u8(addr+1, 0u8);
                        5
                    }
                }
            }
            Instruction(Opcode::CLC, _) => {
                println!("CLC");
                self.p_reg.remove(FLAG_C);
                2
            }
            Instruction(Opcode::XCE, _) => {
                print!("XCE ");
                let e = self.emulation;
                self.emulation = self.p_reg.contains(FLAG_C);

                match e {
                    true => self.p_reg.insert(FLAG_C),
                    false => self.p_reg.remove(FLAG_C),
                }

                match self.emulation {
                    false => {
                        self.p_reg.insert(FLAG_X);
                        self.p_reg.insert(FLAG_M);

                        println!("Enter native mode");
                    }
                    true => {
                        self.p_reg.remove(FLAG_X);
                        self.p_reg.remove(FLAG_M);

                        println!("Enter emulation mode");
                    }
                }

                2
            }
            Instruction(Opcode::Unknown(op), _) => {
                panic!("Unknown instruction: ${:X}", op);
            }
            Instruction(op, val) => {
                panic!("Instruction {:?}, val {:?} unimplemented", op, val);
            }
            _ => unimplemented!()
        }
    }

    pub fn read_u8(&mut self, mem: &Memory) -> u8 {
        let val = mem.peek_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    pub fn read_u16(&mut self, mem: &Memory) -> u16 {
        (self.read_u8(mem) as u16) | ((self.read_u8(mem) as u16) << 8)
    }
}