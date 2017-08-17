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

#[derive(Debug, Clone, Copy)]
pub enum BGMODES {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7,  // The good one.
}

impl Default for BGMODES {
    fn default() -> BGMODES {
        BGMODES::Mode0
    }
}

impl From<u8> for BGMODES {
    fn from(val: u8) -> BGMODES {
        match val & 0b111 {
            0b000 => BGMODES::Mode0,
            0b001 => BGMODES::Mode1,
            0b010 => BGMODES::Mode2,
            0b011 => BGMODES::Mode3,
            0b100 => BGMODES::Mode4,
            0b101 => BGMODES::Mode5,
            0b110 => BGMODES::Mode6,
            0b111 => BGMODES::Mode7,
            _ => BGMODES::Mode0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CHARSIZE {
    S8,
    S16,
}

impl Default for CHARSIZE {
    fn default() -> CHARSIZE {
        CHARSIZE::S8
    }
}

impl From<u8> for CHARSIZE {
    fn from(val: u8) -> CHARSIZE {
        match val & 0b1 {
            0b1 => CHARSIZE::S16,
            _ => CHARSIZE::S8,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BGCHAR(CHARSIZE, CHARSIZE, CHARSIZE, CHARSIZE);

impl From<u8> for BGCHAR {
    fn from(val: u8) -> BGCHAR {
        let bg1 = CHARSIZE::from(val >> 4);
        let bg2 = CHARSIZE::from(val >> 5);
        let bg3 = CHARSIZE::from(val >> 6);
        let bg4 = CHARSIZE::from(val >> 7);
        BGCHAR(bg1, bg2, bg3, bg4)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BGMODE {
    bg_sizes: BGCHAR,
    mode: BGMODES,
}

impl From<u8> for BGMODE {
    fn from(val: u8) -> Self {
        let mode = BGMODES::from(val);
        let size = BGCHAR::from(val);

        Self {
            bg_sizes: size,
            mode: mode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BGSIZE {
    S32x32,
    S64x32,
    S32x64,
    S64x64,
}

impl Default for BGSIZE {
    fn default() -> BGSIZE {
        BGSIZE::S32x32
    }
}

impl From<u8> for BGSIZE {
    fn from(val: u8) -> BGSIZE {
        match val & 0b11 {
            0b00 => BGSIZE::S32x32,
            0b01 => BGSIZE::S64x32,
            0b10 => BGSIZE::S32x64,
            0b11 => BGSIZE::S64x64,
            _ => BGSIZE::S32x32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BGXSC {
    addr: u16,
    size: BGSIZE,
}

impl From<u8> for BGXSC {
    fn from(val: u8) -> Self {
        let addr = ((val & 0xFC) as u16) << 8;
        let size = BGSIZE::from(val);

        Self {
            addr: addr,
            size: size,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BGNBA(u16, u16);

impl From<u8> for BGNBA {
    fn from(val: u8) -> BGNBA {
        let a = ((val as u16) & 0x0F) << 12;
        let b = ((val as u16) & 0xF0) << 4;

        Self {
            0: a,
            1: b,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SCRDES {
    bg1: bool,
    bg2: bool,
    bg3: bool,
    bg4: bool,
    obj: bool,
}

impl From<u8> for SCRDES {
    fn from(val: u8) -> SCRDES {
        let bg1 = (val & 0b00001) == 0b00001;
        let bg2 = (val & 0b00010) == 0b00010;
        let bg3 = (val & 0b00100) == 0b00100;
        let bg4 = (val & 0b01000) == 0b01000;
        let obj = (val & 0b10000) == 0b10000;

        Self {
            bg1: bg1,
            bg2: bg2,
            bg3: bg3,
            bg4: bg4,
            obj: obj,
        }
    }
}
