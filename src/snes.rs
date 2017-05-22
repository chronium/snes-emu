use cart::SnesCart;
use cpu::Ricoh5A22;

use std::cell::RefMut;

#[derive(Clone)]
pub struct SNES {
    cart: SnesCart,
    cpu: Ricoh5A22,
}

impl SNES {
    pub fn new(rom: Vec<u8>) -> SNES {
        SNES {
            cart: SnesCart::new(rom),
            cpu: Ricoh5A22::new(),
        }
    }

    pub fn reset(&mut self) {
        println!("SNES Reset");
        self.cpu.reset(&self.cart);
    }
}

impl From<SNES> for SnesCart {
    fn from(snes: SNES) -> SnesCart {
        snes.cart
    }
}