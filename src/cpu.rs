use snes::SNES;
use cart::SnesCart;

#[derive(Clone)]
pub struct Ricoh5A22 {
    pc: u16
}

impl Ricoh5A22 {
    pub fn new() -> Ricoh5A22 {
        Ricoh5A22 {
            pc: 0,
        }
    }

    pub fn reset(&mut self, cart: &SnesCart) {
        self.pc = (cart[0x7FFC] as u16) | ((cart[0x7FFD] as u16) << 8);
        println!("CPU Reset, PC: {:#X}", self.pc);
    }
}