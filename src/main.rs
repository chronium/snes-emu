#![feature(exclusive_range_pattern)]
#![feature(inclusive_range_syntax)]
#![feature(drop_types_in_const)]
#![feature(integer_atomics)]
#![feature(range_contains)]
#![feature(box_syntax)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;

extern crate minifb;

use std::io::{self, Read, BufRead, Write};
use std::fs::File;

mod cart;
mod snes;
mod inst;
mod scrn;
mod cpu;
mod mem;

use cart::{SnesCart, SnesHeader};
use snes::SNES;
use cpu::Ricoh5A22;
use scrn::Scrn;

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
    println!("Done");

    let stdin = io::stdin();

    let mut bp = Vec::<u16>::new();

    loop {
        print!(">> ");
        io::stdout().flush().expect("Error flushing stdout");

        let mut line = String::new();
        stdin.lock().read_line(&mut line).expect("Error reading from stdin");

        match line.trim() {
            "q" => break,
            "r" => snes.reset(),
            "s" => {
                match snes.step() {
                    Ok(cycles) => { },
                    Err(err) => {
                        println!("{}", err);
                        println!("{:?}", Ricoh5A22::from(snes.clone()));
                    }
                }
            }
            "g" => {
                while !bp.contains(&snes.cpu.pc) && unsafe { Scrn::RUNNING } {
                    match snes.step() {
                        Ok(cycles) => { },
                        Err(err) => {
                            println!("{}", err);
                            println!("{:?}", Ricoh5A22::from(snes.clone()));
                            println!("Instructions ran: {}", snes.step);
                            snes.step = 0;
                            break
                        }
                    } 
                }
                println!("Breakpoint");
            }
            "p" => {
                let cpu = Ricoh5A22::from(snes.clone());
                print!("{:04X}: [", (cpu.stack_ptr() & 0xFFF0));
                for i in (cpu.stack_ptr() & 0xFFF0)...((cpu.stack_ptr() & 0xFFF0) | 0xE) {
                    print!("{:02X} ", cpu.read_u8(&snes.mem, i));
                }
                print!("{:02X}", cpu.read_u8(&snes.mem, ((cpu.stack_ptr() & 0xFFF0) | 0xF)));
                println!("]");
                println!(" {}{:04X}: ^", "   ".repeat((cpu.stack_ptr() & 0xF) as usize), cpu.stack_ptr());
            }
            "c" => println!("{:?}", Ricoh5A22::from(snes.clone())),
            "h" => println!("{:?}", SnesHeader::from(SnesCart::from(snes.clone()))),
            _ => {
                let split: Vec<&str> = line.trim().split(' ').collect();
                match split[0] {
                    "b" => {
                        bp.push(u16::from_str_radix(split[1], 16).unwrap());
                        println!("Breakpoint set at: {}", split[1]);
                    }
                    "m" => {
                        let addr = u16::from_str_radix(split[1], 16).unwrap();
                        let cpu = Ricoh5A22::from(snes.clone());
                        print!("{:04X}: [", (addr & 0xFFF0));
                        for i in (addr & 0xFFF0)...((addr & 0xFFF0) | 0xE) {
                            print!("{:02X} ", cpu.read_u8(&snes.mem, i));
                        }
                        print!("{:02X}", cpu.read_u8(&snes.mem, ((addr & 0xFFF0) | 0xF)));
                        println!("]");
                    }
                    _ => print!("Unknown command: {}", line)
                }
            }
        }
    }
}