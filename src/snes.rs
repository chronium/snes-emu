use cart::{SnesCart, SnesHeader};
use inst::Instruction;
use cpu::Ricoh5A22;
use scrn::{Screen, Scrn};
use mem::Memory;

use std::cell::RefMut;

#[derive(Clone)]
pub struct SNES {
    pub cart: SnesCart,
    pub cpu: Ricoh5A22,
    pub mem: Memory,
    pub step: u64,
}

impl SNES {
    pub fn new(rom: Vec<u8>) -> SNES {
        let cart = SnesCart::new(rom);
        let cpu = Default::default();
        let mem = Memory::new(cart.clone());

        let hdr = SnesHeader::from(cart.clone());

        let screen = Screen::new(hdr.game_title, 256,224);

        unsafe {
            Scrn::SCREEN = Some(screen);
        }

        SNES {
            cart: cart,
            cpu: cpu,
            mem: mem,
            step: 0u64,
        }
    }

    pub fn reset(&mut self) {
        println!("SNES Reset");
        self.cpu.reset(&self.cart);
    }

    pub fn step(&mut self) -> Result<u8, String> {
        self.step += 1;
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