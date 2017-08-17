use inst::{Instruction, Opcode, Value};
use scrn::{Screen, Scrn};
use cart::SnesCart;
use snes::SNES;
use regs::*;
use mem::Memory;

use std::cell::Cell;

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
    pub pc: u16,
    pub p_reg: PReg,
    nmitimen: u8,
    emulation: bool,
    hdmaen: u8,
    mdmaen: u8,
    inidisp: u8,
    fastrom: bool,
    a_reg: u16,
    y_reg: u16,
    x_reg: u16,
    direct_page: u16,
    stack_ptr: u16,
    pbr: u8,
    dbr: u8,
    DMA7: [u8; 7],
    DMA_addr: [u16; 7],
    DMA_size: [u16; 7],
    DMA_bank: [u8; 7],
    DMA_dest: [u8; 7],
    DMAP: [DMAControl; 7],
    VMAIN: VMAIN,
    BGMODE: BGMODE,
    BGXSC: [BGXSC; 4],
    BG12NBA: BGNBA,
    BG34NBA: BGNBA,
    TM: SCRDES,
    TS: SCRDES,
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

        // Set initial register states
        self.a_reg = 0u16;
        self.y_reg = 0u16;
        self.x_reg = 0u16;

        // Set the Direct Page to the Zero Page
        // Emulation mode uses Zero Page here
        self.direct_page = 0u16;

        // Set the Stack Pointer to the First Page
        // Emulation mode uses First page here
        self.stack_ptr = 0x01FFu16;

        // Set the Program Bank Register
        self.pbr = 0u8;

        // Set the Data Bank Register
        self.dbr = 0u8;

        self.VMAIN = Default::default();

        println!("CPU Reset, PC: ${:X}", self.pc);
    }

    pub fn step(&mut self, mem: &mut Memory) -> Result<u8, String> {
        print!("0x{:4X}: ", self.pc);
        match Instruction::from(self, mem) {
            Instruction(Opcode::SEI, _) => {
                println!("SEI");
                // Disable interrupts
                self.p_reg.insert(FLAG_I);
                Ok(2)
            }
            Instruction(Opcode::STZ, Value::Absolute(addr)) => {
                println!("STZ ${:X}", addr);

                // Set zero at location
                match self.p_reg.contains(FLAG_M) {
                    // If 16 bit accumulator write two bytes
                    false => {
                        self.write_u16(mem, addr, 0u16);
                        Ok(5)
                    }
                    // If 8 bit accumulator write one byte
                    true => {
                        self.write_u8(mem, addr, 0u8);
                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::CLC, _) => {
                println!("CLC");
                // Clear Carry Flag
                self.p_reg.remove(FLAG_C);
                Ok(2)
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
                Ok(2)
            }
            Instruction(Opcode::LDA, Value::Immediate8(val)) => {
                println!("LDA #${:X}", val);
                // Load an 8 bit immediate into the A register
                // The A register is the low byte of the 16 bit
                // Whole register. The high byte is B and As whole
                // The register is C, C is 16 bit (B << 8) & A
                self.a_reg = (0xFF00 & self.a_reg) | val as u16;

                // Set the Zero flag
                if val == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if val & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(2)
            }
            Instruction(Opcode::LDA, Value::Immediate16(val)) => {
                println!("LDA #${:X}", val);
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
                if self.a_reg & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(3)
            }
            Instruction(Opcode::REP, Value::Immediate8(flags)) => {
                println!("REP #${:X}", flags);
                // Reset the Processor register bits
                // Based on the immediate
                self.p_reg.bits &= !flags;
                Ok(3)
            }
            Instruction(Opcode::SEP, Value::Immediate8(flags)) => {
                println!("SEP #${:X}", flags);
                // Set the Processor register bits
                // To the immediate value 
                self.p_reg.bits |= flags;
                Ok(3)
            }
            Instruction(Opcode::STA, Value::Absolute(addr)) => {
                println!("STA ${:X}", addr);

                let a = self.a_reg;
                
                // Store A at address
                match self.p_reg.contains(FLAG_M) {
                    // If 16 bit accumulator write the C register
                    false => {
                        self.write_u16(mem, addr, a);

                        // Set the Zero flag
                        if a == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if a & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(5)
                    }
                    // If 8 bit accumulator write the A register
                    true => {
                        self.write_u8(mem, addr, (a & 0xFF) as u8);

                        // Set the Zero flag
                        if (a & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if a & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::TCD, Value::Implied) => {
                println!("TCD");

                // Transfer C register to the Direct Page register
                self.direct_page = self.a_reg;

                // Set the Zero flag
                if self.direct_page == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.direct_page & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(2)
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
                Ok(2)
            }
            Instruction(Opcode::JSR, Value::Absolute(addr)) => {
                println!("JSR ${:X}", addr);

                // Rust borrow crap
                let pc = self.pc;

                // Store the Program Counter
                self.push_u16(mem, pc);

                // Jump!
                self.pc = addr;

                Ok(6)
            }
            Instruction(Opcode::STA, Value::DirectPage(offset)) => {
                println!("STA ${:X}", offset);

                // Store a temporary number of cycles
                let mut cycles = 3;

                // More Rust borrow crap
                let p_reg = self.p_reg;
                let dp = self.direct_page;
                let a = self.a_reg;

                match p_reg.contains(FLAG_M) {
                    // If accumulator is 16 bit write C at
                    // Direct Page + offset
                    false => {
                        self.write_u16(mem,dp + offset as u16, a);
                        // Add one more cycle because we have
                        // Written one more byte
                        cycles += 1;
                    }
                    // If accumulator is 8 bit write A at
                    // Direct Page + offset
                    true => {
                        self.write_u8(mem, dp + offset as u16, (a & 0xFF) as u8);
                    }
                }

                // Add another cycle if we're not reading from
                // Direct Page Zero
                cycles += if dp & 0xFF != 0x00 { 1 } else { 0 };

                Ok(cycles)
            }
            Instruction(Opcode::PHP, Value::Implied) => {
                println!("PHP");

                // Even more Rust borrow crap
                let p = self.p_reg;

                // Push the Processor register
                self.push_u8(mem, p.bits as u8);

                Ok(3)
            }
            Instruction(Opcode::LDY, Value::Immediate16(val)) => {
                println!("LDY #${:X}", val);

                // Load a 16 bit immediate into the Y register
                self.y_reg = val;

                // Set the Zero flag
                if self.y_reg == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.y_reg & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(3)
            }
            Instruction(Opcode::CMP, Value::Absolute(addr)) => {
                println!("CMP ${:X}", addr);

                match self.p_reg.contains(FLAG_M) {
                    // If 8 bit accumulator
                    true => {
                        // Read value from memory
                        let val = self.read_u8(mem, addr, self.dbr);
                        // Compare
                        let res = (self.a_reg & 0xFF) as u8 - val;

                        // Set the Zero flag
                        if (res & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if (self.a_reg & 0xFF) as u8 >= val {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                    // If 16 bit accumulator
                    false => {
                        // Read value from memory
                        let val = self.read_u16(mem, addr, self.dbr);
                        // Compare
                        let res = self.a_reg - val;

                        // Set the Zero flag
                        if res == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if self.a_reg >= val {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                }
                Ok(4)
            }
            Instruction(Opcode::LDX, Value::Immediate16(val)) => {
                println!("LDX #${:X}", val);

                // Load a 16 bit immediate into the X register
                self.x_reg = val;

                // Set the Zero flag
                if self.x_reg == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.x_reg & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(3)
            }
            Instruction(Opcode::LDX, Value::Immediate8(val)) => {
                println!("LDX #${:X}", val);

                // Load a 8 bit immediate into the X register
                self.x_reg = (self.x_reg & 0xFF00) | val as u16;

                // Set the Zero flag
                if val == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if val & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(3)
            }
            Instruction(Opcode::TXS, Value::Implied) => {
                println!("TXS");

                // Transfer X register to Stack register
                self.stack_ptr = self.x_reg;

                // Set the Zero flag
                if self.stack_ptr == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.stack_ptr & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(2)
            }
            Instruction(Opcode::JSR, Value::AbsoluteLong(addr, bank)) => {
                println!("JSL ${:2X}{:4X}", bank, addr);

                // Rust borrow crap, again
                let pbr = self.pbr;
                let pc = self.pc;

                // Store the Program Bank Register
                self.push_u8(mem, pbr);
                // Store the Program Counter
                self.push_u16(mem, pc);
                 
                // Jump!
                self.pc = addr;
                self.pbr = bank;

                Ok(8)
            }
            Instruction(Opcode::PHK, Value::Implied) => {
                println!("PHK");

                // Do you remember this? 
                // Yes! Rust borrow crap! 
                // Once again! Awesome!
                let pbr = self.pbr;

                // Push the Program Bank Register
                self.push_u8(mem, pbr);

                Ok(3)
            }
            Instruction(Opcode::PLB, Value::Implied) => {
                println!("PLB");

                // Pull Data Bank Register
                self.dbr = self.pull_u8(mem);

                // Set the Zero flag
                if self.dbr == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.dbr & 0x80 == 0x80 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(4)
            }
            Instruction(Opcode::LDX, Value::Absolute(addr)) => {
                println!("LDX ${:X}", addr);

                match self.p_reg.contains(FLAG_X) {
                    // If Index registers are 8 bit
                    true => {
                        // Read the value from memory
                        let val = self.read_u8(mem, addr, self.dbr);
                        // Set X register to the value
                        self.x_reg = (self.x_reg & 0xFF00) | val as u16;

                        // Set the Zero flag
                        if val == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if val & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(4)
                    }
                    // If Index registers are 16 bit
                    false => {
                        // Read the value from memory
                        let val = self.read_u16(mem, addr, self.dbr);
                        // Set X register to value
                        self.x_reg = val;

                        // Set the Zero flag
                        if self.x_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(5)
                    }
                }
            }
            Instruction(Opcode::STX, Value::Absolute(addr)) => {
                println!("STX ${:X}", addr);

                // We got Rust again!
                let x = self.x_reg;
                
                // Store X at address
                match self.p_reg.contains(FLAG_X) {
                    // If Index registers are 8 bit
                    true => {
                        self.write_u8(mem, addr, (x & 0xFF) as u8);

                        // Set the Zero flag
                        if (x & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(5)
                    }
                    // If Index registers are 16 bit
                    false => {
                        self.write_u16(mem, addr, x);

                        // Set the Zero flag
                        if self.x_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::STZ, Value::DirectPageX(offset)) => {
                println!("STZ ${:02X},X", offset);

                // Store a temporary number of cycles
                let mut cycles = 4;

                // More Rust borrow crap
                let p_reg = self.p_reg;
                let dp = self.direct_page;
                let x = self.x_reg;

                match p_reg.contains(FLAG_M) {
                    // If memory is 16 bit write Zero at
                    // Direct Page + offset + x
                    false => {
                        self.write_u16(mem,dp + x + offset as u16, 0u16);
                    }
                    // If accumulator is 8 bit write A at
                    // Direct Page + offset
                    true => {
                        self.write_u8(mem, dp + x + offset as u16, 0u8);
                    }
                }

                // Add another cycle if we're not reading from
                // Direct Page Zero
                cycles += if dp & 0xFF != 0x00 { 1 } else { 0 };
                // Add another cycle for 16 bit index registers
                cycles += if p_reg.contains(FLAG_X) { 1 } else { 0 };

                Ok(cycles)
            }
            Instruction(Opcode::INX, Value::Implied) => {
                println!("INX");

                match self.p_reg.contains(FLAG_X) {
                    // For 8 bit Index registers
                    true => {
                        self.x_reg = ((self.x_reg as u8).wrapping_add(1) as u16) & 0xFF;

                        // Set the Zero flag
                        if (self.x_reg & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                    // For 16 bit Index registers
                    false => {
                        self.x_reg = self.x_reg.wrapping_add(1);

                        // Set the Zero flag
                        if self.x_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                }
                Ok(2)
            }
            Instruction(Opcode::XBA, Value::Implied) => {
                println!("XBA");

                // Get the low and high bytes of the A register
                let high = ((self.a_reg & 0xFF00) >> 8) as u8;
                let low  = ((self.a_reg & 0x00FF) >> 0) as u8;

                // Swap the bytes
                self.a_reg = (high as u16) | ((low as u16) << 8);

                // Set the Zero flag
                if (self.a_reg & 0xFF) == 0 {
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

                Ok(3)
            }
            Instruction(Opcode::PHA, Value::Implied) => {
                println!("PHA");

                let a = self.a_reg;

                match self.p_reg.contains(FLAG_M) {
                    // 8 bit Accumulator
                    true => {
                        // Push Accumulator
                        self.push_u8(mem, (a & 0xFF) as u8);
                        Ok(3)
                    }
                    // 16 bit Accumulator
                    false => {
                        // Push Accumulator
                        self.push_u16(mem, a);
                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::PHX, Value::Implied) => {
                println!("PHX");

                let x = self.x_reg;

                match self.p_reg.contains(FLAG_X) {
                    // 8 bit Index registers
                    true => {
                        // Push X
                        self.push_u8(mem, (x & 0xFF) as u8);
                        Ok(3)
                    }
                    // 16 bit Index registers
                    false => {
                        // Push X
                        self.push_u16(mem, x);
                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::PLD, Value::Implied) => {
                println!("PLD");

                // Pull Direct Page Register
                self.direct_page = self.pull_u16(mem);

                // Set the Zero flag
                if self.direct_page == 0 {
                    self.p_reg.insert(FLAG_Z);
                } else {
                    self.p_reg.remove(FLAG_Z);
                }

                // Set the N flag to the most significant bit
                if self.direct_page & 0x8000 == 0x8000 {
                    self.p_reg.insert(FLAG_N);
                } else {
                    self.p_reg.remove(FLAG_N);
                }
                Ok(5)
            }
            Instruction(Opcode::RTS, Value::Implied) => {
                println!("RTS");

                // Pull the address
                let addr = self.pull_u16(mem);

                // Return
                self.pc = addr;

                Ok(6)
            }
            Instruction(Opcode::CPX, Value::Immediate8(val)) => {
                println!("CPX #${:X}", val);

                match self.p_reg.contains(FLAG_X) {
                    // If 8 bit Index registers
                    true => {
                        // Compare
                        let res = ((self.x_reg & 0xFF) as u8) - val;

                        // Set the Zero flag
                        if (res & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if (self.x_reg & 0xFF) as u8 >= val {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(2)
                    }
                    // If 16 bit Index registers
                    false => {
                        // Compare
                        let res = self.x_reg - val as u16;

                        // Set the Zero flag
                        if res == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if self.x_reg >= val as u16 {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(3)                        
                    }
                }
            }
            Instruction(Opcode::CPX, Value::Immediate16(val)) => {
                println!("CPX #${:X}", val);

                match self.p_reg.contains(FLAG_X) {
                    // If 8 bit Index registers
                    true => {
                        // Compare
                        let res = (self.x_reg & 0xFF).wrapping_sub(val);

                        // Set the Zero flag
                        if (res & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if self.x_reg & 0xFF >= val {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(2)
                    }
                    // If 16 bit Index registers
                    false => {
                        // Compare
                        let res = self.x_reg.wrapping_sub(val);

                        // Set the Zero flag
                        if res == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the Carry flag
                        if self.x_reg >= val {
                            self.p_reg.insert(FLAG_C);
                        } else {
                            self.p_reg.remove(FLAG_C);
                        }

                        // Set the N flag
                        if res & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                        Ok(3)                        
                    }
                }
            }
            Instruction(Opcode::BNE, Value::Immediate8(near)) => {
                println!("BNE ${:X}", near);

                let mut cycles = 2;

                // If not Zero
                if !self.p_reg.contains(FLAG_Z) {
                    let pc = self.pc as i32;
                    // Branch relative
                    self.pc = (pc + (near as i8) as i32) as u16;
                    cycles += 1;
                }

                cycles += if self.emulation { 1 } else { 0 };
                Ok(cycles)
            }
            Instruction(Opcode::LDA, Value::Absolute(addr)) => {
                println!("LDA ${:X}", addr);

                match self.p_reg.contains(FLAG_M) {
                    // 8 bit Accumulator
                    true => {
                        // Read value from memory
                        let val = self.read_u8(mem, addr, self.dbr);

                        // Set A register
                        self.a_reg = (self.a_reg & 0xFF00) | val as u16;

                        // Set the Zero flag
                        if (self.a_reg & 0xFF) == 0 {
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

                        Ok(4)
                    }
                    // 16 bit Accumulator
                    false => {
                        let val = self.read_u16(mem, addr, self.dbr);

                        //Set A register
                        self.a_reg = val;

                        // Set the Zero flag
                        if self.a_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.a_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(5)
                    }
                }
            }
            Instruction(Opcode::STX, Value::DirectPage(offset)) => {
                println!("STX ${:X}", offset);

                // Store a temporary number of cycles
                let mut cycles = 3;

                // More Rust borrow crap
                let p_reg = self.p_reg;
                let dp = self.direct_page;
                let x = self.x_reg;

                match p_reg.contains(FLAG_X) {
                    // 8 bit Index registers
                    // Direct Page + offset
                    false => {
                        self.write_u16(mem,dp + offset as u16, x);

                        // Set the Zero flag
                        if x == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if x & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        // Add one more cycle because we have
                        // Written one more byte
                        cycles += 1;
                    }
                    // 16 bit Index registers
                    // Direct Page + offset
                    true => {
                        self.write_u8(mem, dp + offset as u16, (x & 0xFF) as u8);

                        // Set the Zero flag
                        if (x & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if x & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                }

                // Add another cycle if we're not reading from
                // Direct Page Zero
                cycles += if dp & 0xFF != 0x00 { 1 } else { 0 };

                Ok(cycles)
            }
            Instruction(Opcode::PLP, Value::Implied) => {
                println!("PLP");

                // Pull the Processor register from the stack
                self.p_reg = PReg::from_bits(self.pull_u8(mem)).unwrap();

                Ok(4)
            }
            Instruction(Opcode::PLX, Value::Implied) => {
                println!("PLX");

                match self.p_reg.contains(FLAG_X) {
                    // 8 bit Index registers
                    true => {
                        // Pull the value
                        let val = self.pull_u8(mem);

                        // Set low X to the value pulled
                        self.x_reg = (self.x_reg & 0xFF00) | val as u16;

                        // Set the Zero flag
                        if (self.x_reg & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(4)
                    }
                    // 16 bit Index registers
                    false => {
                        // Pull the value
                        let val = self.pull_u16(mem);

                        // Set A to the value pulled
                        self.x_reg = val;

                        // Set the Zero flag
                        if self.x_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(5)
                    }
                }
            }
            Instruction(Opcode::PLA, Value::Implied) => {
                println!("PLA");

                match self.p_reg.contains(FLAG_M) {
                    // 8 bit Accumulator
                    true => {
                        // Pull the value
                        let val = self.pull_u8(mem);

                        // Set low X to the value pulled
                        self.a_reg = (self.a_reg & 0xFF00) | val as u16;

                        // Set the Zero flag
                        if (self.a_reg & 0xFF) == 0 {
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

                        Ok(4)
                    }
                    // 16 bit Accumulator
                    false => {
                        // Pull the value
                        let val = self.pull_u16(mem);

                        // Set A to the value pulled
                        self.a_reg = val;

                        // Set the Zero flag
                        if self.a_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.a_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(5)
                    }
                }
            }
            Instruction(Opcode::DEX, Value::Implied) => {
                println!("DEX");

                match self.p_reg.contains(FLAG_X) {
                    // For 8 bit Index registers
                    true => {
                        self.x_reg -= 1;

                        // Set the Zero flag
                        if (self.x_reg & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                    // For 16 bit Index registers
                    false => {
                        self.x_reg -= 1;

                        // Set the Zero flag
                        if self.x_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.x_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                }
                Ok(2)
            }
            Instruction(Opcode::CLI, Value::Implied) => {
                println!("CLI");

                // Clear Interrupt disable flag
                self.p_reg.remove(FLAG_I);

                Ok(2)
            }
            Instruction(Opcode::RTL, Value::Implied) => {
                println!("RTL");

                // Pop return address
                let addr = self.pull_u16(mem);
                // Pop return Program Bank
                let pbr = self.pull_u8(mem);

                // Perform return
                self.pc = addr;
                self.pbr = pbr;

                Ok(6)
            }
            Instruction(Opcode::JMP, Value::Absolute(addr)) => {
                println!("JMP ${:04X}", addr);

                // Jump!
                self.pc = addr;

                Ok(3)
            }
            Instruction(Opcode::INA, Value::Implied) => {
                println!("INA");

                match self.p_reg.contains(FLAG_M) {
                    // For 8 bit Index registers
                    true => {
                        self.a_reg = ((self.a_reg as u8).wrapping_add(1) as u16) & 0xFF;

                        // Set the Zero flag
                        if (self.a_reg & 0xFF) == 0 {
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
                    }
                    // For 16 bit Index registers
                    false => {
                        self.a_reg = self.a_reg.wrapping_add(1);

                        // Set the Zero flag
                        if self.a_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.a_reg & 0x8000 == 0x8000 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }
                    }
                }
                Ok(2)
            }
            Instruction(Opcode::STY, Value::Absolute(addr)) => {
                println!("STY ${:X}", addr);

                // We got Rust again!
                let y = self.y_reg;
                
                // Store Y at address
                match self.p_reg.contains(FLAG_X) {
                    // If Index registers are 8 bit
                    true => {
                        self.write_u8(mem, addr, (y & 0xFF) as u8);

                        // Set the Zero flag
                        if (y & 0xFF) == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.y_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(5)
                    }
                    // If Index registers are 16 bit
                    false => {
                        self.write_u16(mem, addr, y);

                        // Set the Zero flag
                        if self.y_reg == 0 {
                            self.p_reg.insert(FLAG_Z);
                        } else {
                            self.p_reg.remove(FLAG_Z);
                        }

                        // Set the N flag to the most significant bit
                        if self.y_reg & 0x80 == 0x80 {
                            self.p_reg.insert(FLAG_N);
                        } else {
                            self.p_reg.remove(FLAG_N);
                        }

                        Ok(4)
                    }
                }
            }
            Instruction(Opcode::HLT, _) => {
                Err(String::from("Halt!"))
            }
            Instruction(Opcode::Unknown(op), _) => {
                Err(format!("Unknown instruction: ${:X}", op))
            }
            Instruction(op, val) => {
                Err(format!("Instruction {:?}, val {:?} unimplemented", op, val))
            }
            _ => unimplemented!()
        }
    }

    pub fn read_u8(&self, mem: &Memory, addr: u16, bank: u8) -> u8 {
        match addr {
            0x2000 => 0u8,
            0x4210 => 0x42u8,
            0x2140...0x2143 => 0u8,
            0x4300...0x4306 => 0u8,
            0x4310...0x4316 => 0u8,
            0x4320...0x4326 => 0u8,
            0x4330...0x4336 => 0u8,
            0x4340...0x4346 => 0u8,
            0x4350...0x4356 => 0u8,
            0x4360...0x4366 => 0u8,
            0x4370...0x4376 => self.DMA7[(addr - 0x4370) as usize],
            _ => mem.peek_u8(addr, bank)
        }
    }

    pub fn read_u16(&self, mem: &Memory, addr: u16, bank: u8) -> u16 {
        (self.read_u8(mem, addr, bank) as u16) | ((self.read_u8(mem, addr + 1, bank) as u16) << 8)
    }

    pub fn read_u8_pc(&mut self, mem: &Memory) -> u8 {
        let val = self.read_u8(mem, self.pc, self.pbr);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    pub fn read_u16_pc(&mut self, mem: &Memory) -> u16 {
        (self.read_u8_pc(mem) as u16) | ((self.read_u8_pc(mem) as u16) << 8)
    }

    pub fn push_u8(&mut self, mem: &mut Memory, val: u8) {
        let stack_ptr = self.stack_ptr;
        self.write_u8(mem, stack_ptr, val);
        self.stack_ptr -= 1;
    }

    pub fn push_u16(&mut self, mem: &mut Memory, val: u16) {
        self.push_u8(mem, ((val & 0xFF00) >> 8) as u8);
        self.push_u8(mem, ((val & 0x00FF) >> 0) as u8);
    }

    pub fn pull_u8(&mut self, mem: &Memory) -> u8 {
        self.stack_ptr += 1;
        let stack_ptr = self.stack_ptr;
        self.read_u8(mem, stack_ptr, 0)
    }

    pub fn pull_u16(&mut self, mem: &Memory) -> u16 {
        // You remember our old friend?
        // The Rust borrow checker?
        // Sorry, Rust borrow crap?
        // It's back here!
        let high = self.pull_u8(mem) as u16;
        let low = self.pull_u8(mem) as u16;
        high | (low << 8)
    }

    pub fn write_u8(&mut self, mem: &mut Memory, addr: u16, val: u8) {
        match addr {
            0x2100 => {
                println!("TODO: INIDISP #${:X}", val);
                self.inidisp = val;
            }
            0x2101...0x2104 => {
                println!("TODO: REGISTERS 0x2101...0x2120 ({:X})", addr);
            }
            0x2105 => {
                println!("BGMODE: #${:X}", val);
                self.BGMODE = BGMODE::from(val);
                println!("{:?}", self.BGMODE);
            }
            0x2106 => {
                println!("TODO: REGISTERS 0x2101...0x2120 ({:X})", addr);
            }
            0x2107 => {
                println!("BG1SC: #${:X}", val);
                self.BGXSC[0] = BGXSC::from(val);
                println!("BG1SC: {:?}", self.BGXSC[0]);
            }
            0x2108 => {
                println!("BG2SC: #${:X}", val);
                self.BGXSC[1] = BGXSC::from(val);
                println!("BG2SC: {:?}", self.BGXSC[1]);
            }
            0x2109 => {
                println!("BG3SC: #${:X}", val);
                self.BGXSC[2] = BGXSC::from(val);
                println!("BG3SC: {:?}", self.BGXSC[2]);
            }
            0x210A => {
                println!("BG4SC: #${:X}", val);
                self.BGXSC[3] = BGXSC::from(val);
                println!("BG4SC: {:?}", self.BGXSC[3]);
            }
            0x210B => {
                println!("BG12NBA: #${:X}", val);
                self.BG12NBA = BGNBA::from(val);
                println!("BG12NBA: {:?}", self.BG12NBA);
            }
            0x210C => {
                println!("BG34NBA: #${:X}", val);
                self.BG34NBA = BGNBA::from(val);
                println!("BG34NBA: {:?}", self.BG34NBA);
            }
            0x210D...0x2114 => {
                println!("TODO: REGISTERS 0x2101...0x2120 ({:X})", addr);
            }
            0x2115 => {
                println!("VMAIN: #${:X}", val);
                self.VMAIN = VMAIN::from(val);
                println!("{:?}", self.VMAIN);
            }
            0x2116 => {
                println!("VMADDL: #${:X}", val);
                unsafe { Scrn::VRAM_ADDR = (Scrn::VRAM_ADDR & 0xFF00) | val as u16; }
            }
            0x2117 => {
                println!("VMADDH: #${:X}", val);
                unsafe { Scrn::VRAM_ADDR = (Scrn::VRAM_ADDR & 0x00FF) | ((val as u16) << 8); }
            }
            0x2116...0x2117 => {
                println!("TODO: VMADD (${:X}, #${:X})", addr, val);
            }
            0x2118 => {
                match self.VMAIN.remap {
                    VREMAP::None => {
                        unsafe {
                            Scrn::VRAM[Scrn::VRAM_ADDR as usize] = (Scrn::VRAM[Scrn::VRAM_ADDR as usize] & 0xFF00) | val as u16;
                        
                            if self.VMAIN.increment == VINC::Byte {
                                match self.VMAIN.amount {
                                    VINCAM::One => Scrn::VRAM_ADDR += 1,
                                    VINCAM::ThirtyTwo => Scrn::VRAM_ADDR += 32,
                                    VINCAM::OneTwentyEight => Scrn::VRAM_ADDR += 128,
                                }
                            }
                        }
                    }
                    _ => panic!("VRAM Remap not supported: {:?}", self.VMAIN),
                }
            }
            0x2119 => {
                match self.VMAIN.remap {
                    VREMAP::None => {
                        unsafe {
                            Scrn::VRAM[Scrn::VRAM_ADDR as usize] = (Scrn::VRAM[Scrn::VRAM_ADDR as usize] & 0x00FF) | ((val as u16) << 8);
                        
                            if self.VMAIN.increment == VINC::Word {
                                match self.VMAIN.amount {
                                    VINCAM::One => Scrn::VRAM_ADDR += 1,
                                    VINCAM::ThirtyTwo => Scrn::VRAM_ADDR += 32,
                                    VINCAM::OneTwentyEight => Scrn::VRAM_ADDR += 128,
                                }
                            }
                        }
                    }
                    _ => panic!("VRAM Remap not supported: {:?}", self.VMAIN),
                }
            }
            0x2119..0x2121 => {
                println!("TODO: REGISTERS 0x2101...0x2120 ({:X})", addr);
            }
            0x2121 => {
                println!("CGADD: {:X}", val);
                unsafe {
                    Scrn::PALETTE_INDEX = val as u16 * 2;
                }
            }
            0x2122 => { 
                println!("CGDATA: {:X}", val);
                unsafe {
                    Scrn::PALETTE[Scrn::PALETTE_INDEX as usize] = val;
                    Scrn::PALETTE_INDEX += 1;
                    println!("{:X}", Scrn::PALETTE_INDEX);
                }
            }
            0x2123...0x212B => {
                println!("TODO: REGISTERS 0x2123...0x2133 ({:X})", addr);
            }
            0x212C => {
                println!("TM: #${:X}", val);
                self.TM = SCRDES::from(val);
                println!("TM: {:?}", self.TM);
            }
            0x212D => {
                println!("TS: #${:X}", val);
                self.TS = SCRDES::from(val);
                println!("TS: {:?}", self.TS);
            }
            0x212E...0x2133 => {
                println!("TODO: REGISTERS 0x2123...0x2133 ({:X})", addr);
            }
            0x213E => {
                println!("TODO: STAT77 ${:X}", addr);
            }
            0x2181...0x2183 => {
                println!("TODO: REGISTERS 0x2181...0x2183 ({:X}", addr);
            }
            0x2140...0x2143 => {
                println!("TODO: APUIO #${:X}", addr);
            }
            0x2180 => {
                // TODO: WRAM
            }
            0x4200 => {
                println!("NMITIMEN: #${:X}", val);
                self.nmitimen = val;
            }
            0x4201...0x4203 => {
                println!("TODO: WRIO/WRMPY(A/B) ${:X}", addr);
            }
            0x420B => {
                println!("MDMAEN: #${:X}", val);
                if val & 0b0000001 == 0b0000001 {
                    let control = self.DMAP[0];

                    for i in 0...self.DMA_size[0] {
                        match control.direction {
                            DMADirection::To => {
                                match control.mode {
                                    DMATransferMode::RRW => {
                                        let desta = 0x2100 | self.DMA_dest[0] as u16;
                                        let destb = if desta + 1 > 0x21FF { 0x2100 } else { desta + 1 };
                                        let addr = self.DMA_addr[0];
                                        let a = self.read_u8(mem, addr + 0, self.DMA_bank[0]);
                                        let b = self.read_u8(mem, addr + 1, self.DMA_bank[0]);
                                        self.write_u8(mem, desta, a);
                                        self.write_u8(mem, destb, b);

                                        match control.transfer {
                                            DMATransfer::Fixed => { }
                                            DMATransfer::Adjusted => {
                                                self.DMA_addr[0] += 2;
                                            }
                                        }

                                        if i + 1 >= self.DMA_size[0] / 2 {
                                            break;
                                        }
                                    }
                                    DMATransferMode::RW => {
                                        let dest = 0x2100 | self.DMA_dest[0] as u16;
                                        let a = self.read_u8(mem, self.DMA_addr[0], self.DMA_bank[0]);
                                        self.write_u8(mem, dest, a);

                                        match control.transfer {
                                            DMATransfer::Fixed => { }
                                            DMATransfer::Adjusted => {
                                                self.DMA_addr[0] += 1;
                                            }
                                        }
                                    }
                                    _ => panic!("Unsupported DMA mode: {:?}", control)
                                }
                            }
                            _ => panic!("Unsupported DMA mode: {:?}", control)
                        }
                    }
                }
            }
            0x420C => {
                println!("HDMAEN: #${:X}", val);
                if val != 0 {
                    panic!("HDMAEN: #${:X}", val);
                }
                self.hdmaen = val;
            }
            0x420D => {
                println!("MEMSEL: #${:X}", val);
                self.fastrom = (val & 0b1) == 0b1;
            }
            0x4300...0x4306 => {
                match addr & 0xF {
                    0 => {
                        self.DMAP[0] = DMAControl::from(val);
                    }
                    1 => {
                        println!("BBAD0: {:X}", val);
                        self.DMA_dest[0] = val;
                    }
                    2 => {
                        println!("A1T0L: {:X}", val);
                        self.DMA_addr[0] = (self.DMA_addr[0] & 0xFF00) | val as u16;
                    }
                    3 => {
                        println!("A1T0H: {:X}", val);
                        self.DMA_addr[0] = (self.DMA_addr[0] & 0x00FF) | ((val as u16) << 8);
                    }
                    4 => {
                        println!("T1B0: {:X}", val);
                        self.DMA_bank[0] = val;
                    }
                    5 => {
                        println!("DAS0L: {:X}", val);
                        self.DMA_size[0] = (self.DMA_size[0] & 0xFF00) | val as u16;
                    }
                    6 => {
                        println!("DAS0H: {:X}", val);
                        self.DMA_size[0] = (self.DMA_size[0] & 0x00FF) | ((val as u16) << 8);
                    }
                    _ => panic!("DMA: {:X}", addr & 0xF)
                }
            }
            0x4310...0x4316 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4320...0x4326 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4330...0x4336 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4340...0x4346 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4350...0x4356 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4360...0x4366 => {
                println!("TODO: DMA ${:X}", addr);
            }
            0x4370...0x4376 => self.DMA7[(addr - 0x4370) as usize] = val,
            0x4372 => {
                println!("TODO: A1Bx - DMA Source Address ${:X}", addr);
            }
            0x4373 => {
                println!("TODO: A1Bx - DMA Source Address ${:X}", addr);
            }
            0x4374 => {
                println!("TODO: A1Bx - DMA Source Address ${:X}", addr);
            }
            0x4375 => {
                println!("TODO: A1Bx - DMA Source Address ${:X}", addr);
            }
            _ => mem.write_u8(addr, val)
        }
    }

    pub fn write_u16(&mut self, mem: &mut Memory, addr: u16, val: u16) {
        self.write_u8(mem, addr + 0, ((val & 0x00FF) >> 0) as u8);
        self.write_u8(mem, addr + 1, ((val & 0xFF00) >> 8) as u8);
    }

    pub fn stack_ptr(&self) -> u16 {
        self.stack_ptr
    }
}