use std::ops::Index;

#[derive(Debug, Clone)]
pub struct SnesHeader {
    pub game_title: String,
    makeup_byte: u8,
    rom_type: u8,
    rom_size: usize,
    sram_size: usize,
    license_id: [u8;2],
    version: u8,
    check_compl: u8,
    checksum: [u8;2],
}

#[derive(Clone)]
pub struct SnesCart {
    cart_data: Vec<u8>,
    rom_length: usize,
    header: SnesHeader,
}

impl From<Vec<u8>> for SnesHeader {
    fn from(cart: Vec<u8>) -> Self {
        let mut title = [0u8; 21];
        for i in 0..21 {
            title[i] = cart[i + 0x7FC0];
        }

        let mut license = [0u8; 2];
        license[0] = cart[0x7FD9];
        license[1] = cart[0x7FDA];

        let mut checksum = [0u8; 2];
        checksum[0] = cart[0x7FDE];
        checksum[1] = cart[0x7FDF];

        SnesHeader {
            game_title:  String::from_utf8(title.to_vec()).unwrap(),
            makeup_byte: cart[0x7FD5],
            rom_type:    cart[0x7FD6],
            rom_size:    0x400 << cart[0x7FD7],
            sram_size:   0x400 << cart[0x7FD8],
            license_id:  license,
            version:     cart[0x7FDB],
            check_compl: cart[0x7FDC],
            checksum:    checksum,
        }
    }
}

impl Index<usize> for SnesCart {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.cart_data[index]
    }
}

impl SnesCart {
    pub fn new(rom: Vec<u8>) -> SnesCart {
        SnesCart {
            cart_data: rom.clone(),
            rom_length: rom.len(),
            header: SnesHeader::from(rom),
        }
    }
}

impl From<SnesCart> for SnesHeader {
    fn from(cart: SnesCart) -> SnesHeader {
        cart.header
    }
}