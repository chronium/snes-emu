#![feature(exclusive_range_pattern)]
#![feature(inclusive_range_syntax)]
#![feature(drop_types_in_const)]
#![feature(integer_atomics)]
#![feature(range_contains)]
#![feature(relaxed_adts)]
#![feature(box_syntax)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;

extern crate clock_ticks;
extern crate minifb;

use std::io::{self, Read, BufRead, Write};
use std::fs::File;

mod cart;
mod snes;
mod inst;
mod scrn;
mod regs;
mod cpu;
mod mem;

use cart::{SnesCart, SnesHeader};
use snes::SNES;
use cpu::Ricoh5A22;
use scrn::{Scrn, Screen};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub fn cpu_loop<F>(rate: u64, mut callback: F)
    where F: FnMut() -> State + Send + 'static
{
    thread::spawn(move || {
        let mut accumulator = 0;
        let mut previous_clock = clock_ticks::precise_time_ns();

        let rate = 1_000_000_000 / rate;

        loop {
            match callback() {
                State::Stop => break,
                State::Continue => (),
            };

            let now = clock_ticks::precise_time_ns();
            accumulator += now - previous_clock;
            previous_clock = now;

            while accumulator >= rate {
                accumulator -= rate;
            }

            thread::sleep(Duration::from_millis(((rate - accumulator) / 1000000) as u64));
        }
    });
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum State {
    Continue,
    Stop,
}

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

    std::thread::spawn(move || {
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
                        print!("{:02X} ", cpu.read_u8(&snes.mem, i, 0u8));
                    }
                    print!("{:02X}", cpu.read_u8(&snes.mem, ((cpu.stack_ptr() & 0xFFF0) | 0xF), 0u8));
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
                                print!("{:02X} ", cpu.read_u8(&snes.mem, i, 0u8));
                            }
                            print!("{:02X}", cpu.read_u8(&snes.mem, ((addr & 0xFFF0) | 0xF), 0u8));
                            println!("]");
                        }
                        "vm" => {
                            let addr = u16::from_str_radix(split[1], 16).unwrap();
                            print!("{:04X}: [", (addr & 0xFFF0));
                            for i in (addr & 0xFFF0)...((addr & 0xFFF0) | 0xE) {
                                unsafe { print!("{:04X} ", Scrn::VRAM[i as usize]); }
                            }
                            unsafe { print!("{:04X}", Scrn::VRAM[((addr & 0xFFF0) | 0xF) as usize]); }
                            println!("]");
                        }
                        "vc" => {
                            let addr = u16::from_str_radix(split[1], 16).unwrap();
                            print!("{:04X}: [", (addr & 0xFFF0));
                            for i in (addr & 0xFFF0)...((addr & 0xFFF0) | 0xE) {
                                unsafe { print!("{:02X} ", Scrn::PALETTE[i as usize]); }
                            }
                            unsafe { print!("{:02X}", Scrn::VRAM[((addr & 0xFFF0) | 0xF) as usize]); }
                            println!("]");
                        }
                        _ => print!("Unknown command: {}", line)
                    }
                }
            }
        }
    });

    let screen = Screen::new(String::from("snes-emu"), 256,224);

    unsafe {
        Scrn::SCREEN = Some(screen);
    }
}