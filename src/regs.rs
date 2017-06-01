#[derive(Debug, Clone, Copy)]
pub enum DMATransferMode {
    RW,     // 1 register write once
    RRW,    // 2 registers write once
    RWW,    // 1 register write twice
    RRWW,   // 2 registers write twice each 
    RRRRW,  // 4 registers write once
    RWRW,   // 2 registers write twice alternate
}

impl Default for DMATransferMode {
    fn default() -> DMATransferMode {
        DMATransferMode::RW
    }
}

impl From<u8> for DMATransferMode {
    fn from(val: u8) -> DMATransferMode {
        match val & 0b111 {
            0b000 => DMATransferMode::RW,
            0b001 => DMATransferMode::RRW,
            0b010 => DMATransferMode::RWW,
            0b011 => DMATransferMode::RRWW,
            0b100 => DMATransferMode::RRRRW,
            0b101 => DMATransferMode::RWRW,
            0b110 => DMATransferMode::RRW,
            0b111 => DMATransferMode::RRWW,
            _ => panic!("WTF!")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DMADirection {
    To,
    From,
}

impl Default for DMADirection {
    fn default() -> DMADirection {
        DMADirection::To
    }
}

impl From<u8> for DMADirection {
    fn from(val: u8) -> DMADirection {
        match val & 0b10000000 {
            0b10000000 => DMADirection::From,
            _ => DMADirection::To,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DMATransfer {
    Adjusted,
    Fixed,
}

impl Default for DMATransfer {
    fn default() -> DMATransfer {
        DMATransfer::Adjusted
    }
}

impl From<u8> for DMATransfer {
    fn from(val: u8) -> DMATransfer {
        match val & 0b00001000 {
            0b00001000 => DMATransfer::Fixed,
            _ => DMATransfer::Adjusted,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DMAIncrement {
    Increment,
    Decrement,
}

impl Default for DMAIncrement {
    fn default() -> DMAIncrement {
        DMAIncrement::Increment
    }
}

impl From<u8> for DMAIncrement {
    fn from(val: u8) -> DMAIncrement {
        match val & 0b00010000 {
            0b00010000 => DMAIncrement::Decrement,
            _ => DMAIncrement::Increment,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HDMAAddressing {
    Direct,
    Indirect,
}

impl Default for HDMAAddressing {
    fn default() -> HDMAAddressing {
        HDMAAddressing::Direct
    }
}

impl From<u8> for HDMAAddressing {
    fn from(val: u8) -> HDMAAddressing {
        match val & 0b01000000 {
            0b01000000 => HDMAAddressing::Indirect,
            _ => HDMAAddressing::Direct,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DMAControl {
    pub transfer: DMATransfer,
    pub increment: DMAIncrement,
    pub hdma_mode: HDMAAddressing,
    pub direction: DMADirection,
    pub mode: DMATransferMode,
}

impl From<u8> for DMAControl {
    fn from(val: u8) -> Self {
        let mode = DMATransferMode::from(val);
        let direction = DMADirection::from(val);
        let transfer = DMATransfer::from(val);
        let increment = DMAIncrement::from(val);
        let addressing = HDMAAddressing::from(val);

        Self {
            transfer: transfer,
            increment: increment,
            hdma_mode: addressing,
            direction: direction,
            mode: mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VINC {
    Byte,
    Word,
}

impl Default for VINC {
    fn default() -> VINC {
        VINC::Byte
    }
}

impl From<u8> for VINC {
    fn from(val: u8) -> VINC {
        match val & 0b10000000 {
            0b00000000 => VINC::Byte,
            0b10000000 => VINC::Word,
            _ => VINC::Byte,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VINCAM {
    One,
    ThirtyTwo,
    OneTwentyEight,
}

impl Default for VINCAM {
    fn default() -> VINCAM {
        VINCAM::One
    }
}

impl From<u8> for VINCAM {
    fn from(val: u8) -> VINCAM {
        match val & 0b00000011 {
            0b00000000 => VINCAM::One,
            0b00000001 => VINCAM::ThirtyTwo,
            _ => VINCAM::OneTwentyEight,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VREMAP {
    None,
    First,  // aaaaaaaaBBBccccc => aaaaaaaacccccBBB
    Second, // aaaaaaaBBBcccccc => aaaaaaaccccccBBB
    Third,  // aaaaaaBBBccccccc => aaaaaacccccccBBB
}

impl Default for VREMAP {
    fn default() -> VREMAP {
        VREMAP::None
    }
}

impl From<u8> for VREMAP {
    fn from(val: u8) -> VREMAP {
        match val & 0b00001100 {
            0b00000000 => VREMAP::None,
            0b00000100 => VREMAP::First,
            0b00001000 => VREMAP::Second,
            0b00001100 => VREMAP::Third,
            _ => VREMAP::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VMAIN {
    pub increment: VINC,
    pub amount: VINCAM,
    pub remap: VREMAP
}

impl From<u8> for VMAIN {
    fn from(val: u8) -> Self {
        let inc = VINC::from(val);
        let amnt = VINCAM::from(val);
        let remap = VREMAP::from(val);

        Self {
            increment: inc,
            amount: amnt,
            remap: remap,
        }
    }
}
