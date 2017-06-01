use cart::SnesCart;

use std::cell::Cell;

#[derive(Clone)]
pub struct Memory {
    cart: SnesCart,
    wram: Cell<[u8; 0x2000]>,
}

impl Memory {
    pub fn new(cart: SnesCart) -> Memory {
        Memory {
            cart: cart,
            wram: Cell::new([0x55u8; 0x2000]),
        }
    }

    pub fn peek_u8(&self, addr: u16, bank: u8) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000...0x1FFF => self.wram.get()[addr],
            0x8000...0xFFFF => self.cart[(addr - 0x8000) + bank as usize * 0x8000],
            _ => panic!("Unsupported memory read at: ${:X}", addr)
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000...0x1FFF => self.wram.get_mut()[addr] = val,
            _ => panic!("Unsupported memory write at: ${:X} with value: ${:X}", addr, val)
        }
    }
}