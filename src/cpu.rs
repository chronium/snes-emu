use inst::{Instruction, Opcode, Value};
use cart::SnesCart;
use mem::Memory;
use snes::SNES;

bitflags! {
    pub flags PReg: u8 {
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
    pub p_reg: PReg,
    nmitimen: u8,
    emulation: bool,
    hdmaen: u8,
    mdmaen: u8,
    inidisp: u8,
    fastrom: bool,
    a_reg: u16,
}

impl Ricoh5A22 {
    pub fn new() -> Ricoh5A22 {
        Ricoh5A22 {
            pc:         0u16,
            p_reg:      PReg::empty(),
            nmitimen:   0u8,
            emulation:  true,
            hdmaen:     0u8,
            mdmaen:     0u8,
            inidisp:    0u8,
            fastrom:    false,
            a_reg:      0u16,
        }
    }

    pub fn reset(&mut self, cart: &SnesCart) {
        self.pc = (cart[0x7FFC] as u16) | ((cart[0x7FFD] as u16) << 8);
        self.p_reg.insert(FLAG_M);
        self.emulation = true;
        self.hdmaen = 0u8;
        self.mdmaen = 0u8;
        self.inidisp = 0u8;
        self.fastrom = false;
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
                    false => {
                        self.write_u8(mem, addr, 0u8);
                        self.write_u8(mem, addr+1, 0u8);
                        5
                    }
                    true => {
                        self.write_u8(mem, addr, 0u8);
                        4
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
                        println!("Enter native mode");
                    }
                    true => {
                        println!("Enter emulation mode");
                    }
                }
                2
            }
            Instruction(Opcode::LDA, Value::Immediate8(val)) => {
                println!("LDA #${:X}", val);
                self.a_reg = val as u16;
                2
            }
            Instruction(Opcode::LDA, Value::Immediate16(val)) => {
                println!("LDA #${:X}", val);
                self.a_reg = val;
                3
            }
            Instruction(Opcode::REP, Value::Immediate8(flags)) => {
                println!("REP #${:X}", flags);
                self.p_reg.bits &= !flags;
                3
            }
            Instruction(Opcode::SEP, Value::Immediate8(flags)) => {
                println!("SEP #${:X}", flags);
                self.p_reg.bits |= !flags;
                3
            }
            Instruction(Opcode::STA, Value::Absolute(addr)) => {
                println!("STA ${:X}", addr);
                
                match self.p_reg.contains(FLAG_M) {
                    false => {
                        let a = self.a_reg;
                        self.write_u8(mem, addr + 0, (a & 0xFF) as u8);
                        self.write_u8(mem, addr + 1, ((a & 0xFF00) >> 8) as u8);
                        5
                    }
                    true => {
                        let a = self.a_reg;
                        self.write_u8(mem, addr + 0, (a & 0xFF) as u8);
                        4
                    }
                }
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

    pub fn write_u8(&mut self, mem: &Memory, addr: u16, val: u8) {
        match addr {
            0x2100 => {
                println!("TODO: INIDISP #${:X}", val);
                self.inidisp = val;
            }
            0x2140...0x2143 => {
                println!("TODO: APUIO #${:X}", addr);
            }
            0x4200 => {
                println!("NMITIMEN: #${:X}", val);
                self.nmitimen = val;
            }
            0x420B => {
                println!("MDMAEN: #${:X}", val);
                self.mdmaen = val;
            }
            0x420C => {
                println!("HDMAEN: #${:X}", val);
                self.hdmaen = val;
            }
            0x420D => {
                println!("MEMSEL: #${:X}", val);
                self.fastrom = (val & 0b1) == 0b1;
            }
            _ => mem.write_u8(addr, val)
        }
    }
}