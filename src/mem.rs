use cart::SnesCart;

#[derive(Clone)]
pub struct Memory {
    cart: SnesCart,
}

impl Memory {
    pub fn new(cart: SnesCart) -> Memory {
        Memory {
            cart: cart,
        }
    }

    pub fn peek_u8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x8000...0xFFFF => self.cart[addr - 0x8000],
            _ => panic!("Unsupported memory read at: ${:X}", addr)
        }
    }

    pub fn write_u8(&self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            _ => panic!("Unsupported memory write at: ${:X} with value: ${:X}", addr, val)
        }
    }
}