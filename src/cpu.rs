use inst::{Instruction, Opcode, Value};
use cart::SnesCart;
use mem::Memory;
use snes::SNES;

bitflags! {
    #[derive(Default)]
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

#[derive(Debug, Clone, Default)]
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
    direct_page: u8,
    stack_ptr: u16,
    pbr: u8,
}

impl Ricoh5A22 {
    pub fn reset(&mut self, cart: &SnesCart) {
        // Reset the CPU
        // Set the Program Counter to the Reset Vector
        self.pc = (cart[0x7FFC] as u16) | ((cart[0x7FFD] as u16) << 8);

        // Set 8 bit accumulator mode
        self.p_reg.insert(FLAG_M);

        // Set emulation mode
        self.emulation = true;

        // Get rid of any HDMA
        self.hdmaen = 0u8;

        // Get rid of any MDMA
        self.mdmaen = 0u8;

        // Set the display off with 0 brightness
        self.inidisp = 0u8;

        // We're in slow ROM territory
        self.fastrom = false;

        // Set the Direct Page to the Zero Page
        // Emulation mode uses Zero Page here
        self.direct_page = 0u8;

        // Set the Stack Pointer to the First Page
        // Emulation mode uses First page here
        self.stack_ptr = 0x1000u16;

        // Set the Program Bank Register
        self.pbr = 0u8;

        println!("CPU Reset, PC: ${:X}", self.pc);
    }

    pub fn step(&mut self, mem: &Memory) -> u64 {
        print!("0x{:4X}: ", self.pc);
        match Instruction::from(self, mem) {
            Instruction(Opcode::SEI, _) => {
                println!("SEI");
                // Disable interrupts
                self.p_reg.insert(FLAG_I);
                2
            }
            Instruction(Opcode::STZ, Value::Absolute(addr)) => {
                println!("STZ ${:X}", addr);

                // Set zero at location
                match self.p_reg.contains(FLAG_M) {
                    // If 16 bit accumulator write two bytes
                    false => {
                        self.write_u8(mem, addr, 0u8);
                        self.write_u8(mem, addr+1, 0u8);
                        5
                    }
                    // If 8 bit accumulator write one byte
                    true => {
                        self.write_u8(mem, addr, 0u8);
                        4
                    }
                }
            }
            Instruction(Opcode::CLC, _) => {
                println!("CLC");
                // Clear Carry Flag
                self.p_reg.remove(FLAG_C);
                2
            }
            Instruction(Opcode::XCE, _) => {
                print!("XCE ");
                // Exchange Carry with Emulation flag
                let e = self.emulation;
                self.emulation = self.p_reg.contains(FLAG_C);

                // Set the carry flag to the old Emulation flag
                match e {
                    true => self.p_reg.insert(FLAG_C),
                    false => self.p_reg.remove(FLAG_C),
                }

                // You can read yourself, if it's false, 
                // No emulation, otherwise we're in 6502
                // Emulation mode
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
                // Load an 8 bit immediate into the A register
                // The A register is the low byte of the 16 bit
                // Whole register. The high byte is B and As whole
                // The register is C, C is 16 bit (B << 8) & A
                self.a_reg = val as u16;

                // Set the Zero flag
                if self.a_reg == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.a_reg & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                2
            }
            Instruction(Opcode::LDA, Value::Immediate16(val)) => {
                println!("LDC #${:X}", val);
                // Load a 16 bit immediate into the C register
                // But for easy reference, I call it A
                self.a_reg = val;

                // Set the Zero flag
                if self.a_reg == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.a_reg & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                3
            }
            Instruction(Opcode::REP, Value::Immediate8(flags)) => {
                println!("REP #${:X}", flags);
                // Reset the Processor register bits
                // Based on the immediate
                self.p_reg.bits &= !flags;
                3
            }
            Instruction(Opcode::SEP, Value::Immediate8(flags)) => {
                println!("SEP #${:X}", flags);
                // Set the Processor register bits
                // To the immediate value 
                self.p_reg.bits |= !flags;
                3
            }
            Instruction(Opcode::STA, Value::Absolute(addr)) => {
                println!("STA ${:X}", addr);
                
                // Store A at address
                match self.p_reg.contains(FLAG_M) {
                    // If 16 bit accumulator write the C register
                    false => {
                        let a = self.a_reg;
                        self.write_u8(mem, addr + 0, (a & 0xFF) as u8);
                        self.write_u8(mem, addr + 1, ((a & 0xFF00) >> 8) as u8);
                        5
                    }
                    // If 8 bit accumulator write the A register
                    true => {
                        let a = self.a_reg;
                        self.write_u8(mem, addr + 0, (a & 0xFF) as u8);
                        4
                    }
                }
            }
            Instruction(Opcode::TCD, Value::Implied) => {
                println!("TCD");

                // Transfer C register to the Direct Page register
                self.direct_page = (self.a_reg & 0xFF) as u8;

                // Set the Zero flag
                if self.direct_page == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.direct_page & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                2
            }
            Instruction(Opcode::TCS, Value::Implied) => {
                println!("TCS");

                match self.p_reg.contains(FLAG_M) {
                    // If 16 bit accumulator write the C register
                    false => {
                        self.stack_ptr = self.a_reg;
                    }
                    // If 8 bit accumulator write the A register
                    // And remain in page 1
                    true => {
                        self.stack_ptr = 0x1000 + (self.a_reg & 0xFF);
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