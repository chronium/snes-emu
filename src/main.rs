#[macro_use]
extern crate clap;

use std::io::{self, Read, BufRead, Write};
use std::fs::File;

mod cart;
mod snes;
mod cpu;

use cart::{SnesCart, SnesHeader};
use snes::SNES;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = clap_app!(snes_emu =>
        (version: VERSION)
        (author: AUTHORS)
        (about: "SNES Emulator written in Rust")
        (@arg INPUT: +required "Sets the ROM file to emulate")
    ).get_matches();

    let rom_path = matches.value_of("INPUT").unwrap();
    println!("Opening ROM: {}", rom_path);

    let mut rom_raw = Vec::<u8>::new();

    let mut file = match File::open(rom_path) {
        Ok(f) => f,
        Err(err) => panic!("Could not open ROM: {}", err)
    };

    println!("Reading ROM");
    file.read_to_end(&mut rom_raw).unwrap();

    let mut snes = SNES::new(rom_raw);

    println!("{:?}", SnesHeader::from(SnesCart::from(snes.clone())));

    let stdin = io::stdin();
    loop {
        print!(">> ");
        io::stdout().flush().expect("Error flushing stdout");

        let mut line = String::new();
        stdin.lock().read_line(&mut line).expect("Error reading from stdin");

        match line.trim() {
            "q" => break,
            "r" => snes.reset(),
            _ => {}
        }
    }
}