use inst::Instruction;
use cart::SnesCart;
use cpu::Ricoh5A22;
use mem::Memory;

use std::cell::RefMut;

#[derive(Clone)]
pub struct SNES {
    pub cart: SnesCart,
    pub cpu: Ricoh5A22,
    pub mem: Memory,
}

impl SNES {
    pub fn new(rom: Vec<u8>) -> SNES {
        let cart = SnesCart::new(rom);
        let cpu = Default::default();
        let mem = Memory::new(cart.clone());

        SNES {
            cart: cart,
            cpu: cpu,
            mem: mem,
        }
    }

    pub fn reset(&mut self) {
        println!("SNES Reset");
        self.cpu.reset(&self.cart);
    }

    pub fn step(&mut self) -> Result<u8, String> {
        self.cpu.step(&mut self.mem)
    }
}

impl From<SNES> for SnesCart {
    fn from(snes: SNES) -> SnesCart {
        snes.cart
    }
}

impl From<SNES> for Ricoh5A22 {
    fn from(snes: SNES) -> Ricoh5A22 {
        snes.cpu
    }
}