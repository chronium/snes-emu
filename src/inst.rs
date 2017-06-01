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
    DirectPage(u8),
    DirectPageX(u8),
    AbsoluteLong(u16, u8),
}

macro_rules! implied {
    ($instr:ident) => (Instruction(Opcode::$instr, Value::Implied))
}

macro_rules! absolute {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::Absolute($cpu.read_u16_pc($mem))))
}

macro_rules! immediate_m {
    ($instr:ident, $cpu:ident, $mem:ident) => (match $cpu.p_reg.contains(FLAG_M) {
        true => Instruction(Opcode::$instr, Value::Immediate8($cpu.read_u8_pc($mem))),
        false => Instruction(Opcode::$instr, Value::Immediate16($cpu.read_u16_pc($mem))),
    })
}

macro_rules! immediate_x {
    ($instr:ident, $cpu:ident, $mem:ident) => (match $cpu.p_reg.contains(FLAG_X) {
        true => Instruction(Opcode::$instr, Value::Immediate8($cpu.read_u8_pc($mem))),
        false => Instruction(Opcode::$instr, Value::Immediate16($cpu.read_u16_pc($mem))),
    })
}

macro_rules! immediate8 {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::Immediate8($cpu.read_u8_pc($mem))))
}

macro_rules! direct_page {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::DirectPage($cpu.read_u8_pc($mem))))
}

macro_rules! absolute_long {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::AbsoluteLong($cpu.read_u16_pc($mem), $cpu.read_u8_pc($mem))))
}

macro_rules! direct_page_x {
    ($instr:ident, $cpu:ident, $mem:ident) => (Instruction(Opcode::$instr, Value::DirectPageX($cpu.read_u8_pc($mem))))
}

#[derive(Debug)]
pub enum Opcode {
    PHP,        //                         08
    CLC,        //                         18
    INA,        //                            1A
    TCS,        //                                  1B
    JSR,        // 20     22
    PLP,        //                         28
    PLD,        //                                  2B
    PHA,        //                         48
    PHK,        //                                  4B
    JMP,        //                                     4C
    CLI,        //                         58
    TCD,        //                                  5B
    RTS,        // 60
    PLA,        //                         68
    RTL,        //                                  6B
    SEI,        //                         78
    STA,        //                85                      8D
    STX,        //                   86                      8E
    TXS,        //                               9A 
    STZ,        //             74 9C
    LDY,        // A0
    LDX,        //       A2                                  AE
    LDA,        //                            A9          AD
    PLB,        //                                  AB    
    REP,        //       C2
    DEX,        //                               CA   
    CMP,        //                                        CD
    SEP,        //       E2
    BNE,        // D0
    PHX,        //                               DA
    CPX,        // E0
    INX,        //                         E8    
    XBA,        //                                  EB
    PLX,        //                               FA
    XCE,        //                                  FB
    Unknown(u8), 
}

pub struct Instruction(pub Opcode, pub Value);

impl Instruction {
    pub fn from(cpu: &mut Ricoh5A22, mem: &Memory) -> Instruction {
        match cpu.read_u8_pc(mem) {
            0x08 => implied!(PHP),                                  // 0x08 PHP
            0x18 => implied!(CLC),                                  // 0x18 CLC
            0x1A => implied!(INA),                                  // 0x1A INA
            0x1B => implied!(TCS),                                  // 0x1B TCS/TAS
            0x20 => absolute!(JSR, cpu, mem),                       // 0x20 JSR addr
            0x22 => absolute_long!(JSR, cpu, mem),                  // 0x22 JSR long
            0x28 => implied!(PLP),                                  // 0x28 PLP
            0x2B => implied!(PLD),                                  // 0x2B PLD
            0x48 => implied!(PHA),                                  // 0x48 PHA
            0x4B => implied!(PHK),                                  // 0x4B PHK
            0x4C => absolute!(JMP, cpu, mem),                       // 0x4C JMP addr
            0x58 => implied!(CLI),                                  // 0x58 CLI
            0x5B => implied!(TCD),                                  // 0x5B TCD/TAD
            0x60 => implied!(RTS),                                  // 0x60 RTS
            0x68 => implied!(PLA),                                  // 0x68 PLA
            0x6B => implied!(RTL),                                  // 0x6B RTL
            0x74 => direct_page_x!(STZ, cpu, mem),                  // 0x74 STZ dp,X
            0x78 => implied!(SEI),                                  // 0x78 SEI
            0x85 => direct_page!(STA, cpu, mem),                    // 0x85 STA dp
            0x86 => direct_page!(STX, cpu, mem),                    // 0x86 STX dp
            0x8D => absolute!(STA, cpu, mem),                       // 0x8D STA addr
            0x8E => absolute!(STX, cpu, mem),                       // 0x8E STX addr
            0x9A => implied!(TXS),                                  // 0x9A TXS
            0x9C => absolute!(STZ, cpu, mem),                       // 0x9C STZ addr
            0xA0 => immediate_x!(LDY, cpu, mem),                    // 0xA0 LDY #const
            0xA2 => immediate_x!(LDX, cpu, mem),                    // 0xA2 LDX #const
            0xA9 => immediate_m!(LDA, cpu, mem),                    // 0xA9 LDA #const
            0xAB => implied!(PLB),                                  // 0xAB PLB
            0xAD => absolute!(LDA, cpu, mem),                       // 0xAD LDA addr
            0xAE => absolute!(LDX, cpu, mem),                       // 0xAE LDX addr
            0xC2 => immediate8!(REP, cpu, mem),                     // 0xC2 REP #const
            0xCA => implied!(DEX),                                  // 0xCA DEX
            0xCD => absolute!(CMP, cpu, mem),                       // 0xCD CMP addr
            0xD0 => immediate8!(BNE, cpu, mem),                     // 0xD0 BNE near
            0xDA => implied!(PHX),                                  // 0xDA PHX
            0xE0 => immediate_x!(CPX, cpu, mem),                    // 0xE0 CPX #const
            0xE2 => immediate8!(SEP, cpu, mem),                     // 0xE2 SEP #const
            0xE8 => implied!(INX),                                  // 0xE8 INX
            0xEB => implied!(XBA),                                  // 0xEB XBA
            0xFA => implied!(PLX),                                  // 0xFA PLX
            0xFB => implied!(XCE),                                  // 0xFB XCE
            op => Instruction(Opcode::Unknown(op), Value::Implied),
        }
    }
}