use std::sync::atomic::{AtomicU8};
use std::sync::{Mutex, Arc};
use std::cell::{Cell, UnsafeCell};
use std::time::Duration;
use std::thread;

use clock_ticks;

use minifb::{Key, Window, WindowOptions, Scale};

pub mod Scrn {
    pub static mut SCREEN: Option<super::Screen> = None;
    pub static mut PALETTE_INDEX: u16 = 0u16;
    pub static mut PALETTE: [u8; 1024] = [0u8; 1024];
    pub static mut RUNNING: bool = false;
    pub static mut VRAM: [u16; 0x10000] = [0u16; 0x10000];
    pub static mut VRAM_ADDR: u16 = 0u16;
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum State {
    Continue,
    Stop,
}

#[derive(Clone)]
pub struct Screen {
    width: usize,
    height: usize,
}

#[inline]
pub fn get_color(color: u16) -> u32 {
    /*let r = color & 0b000000000011111;
    let g = color & 0b000001111100000;
    let b = color & 0b111110000000000;
    let r = r >> 0;
    let g = g >> 5;
    let b = b >> 10;
    let r = (r * 255) / 31;
    let g = (g * 255) / 31;
    let b = (b * 255) / 31;

    let r = ((r as u32) << 16) & 0xFF0000;
    let g = ((g as u32) << 8) & 0x00FF00;
    let b = ((b as u32) << 0) & 0x0000FF;*/

    (((((((color & 0b0000000000011111) >> 00) * 255) / 31) as u32) << 16) & 0xFF0000) | 
    (((((((color & 0b0000001111100000) >> 05) * 255) / 31) as u32) << 08) & 0x00FF00) |
    (((((((color & 0b0111110000000000) >> 10) * 255) / 31) as u32) << 00) & 0x0000FF)
}

pub fn draw_loop<F>(rate: u64, mut callback: F) where 
    F: FnMut() -> State {
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
}

impl Screen {
    pub fn new_scaled(title: String, width: usize, height: usize, scale: Scale) -> Screen {

        unsafe { Scrn::RUNNING = true; }


        let mut window = Window::new(&title, width, height, WindowOptions {
            scale: scale.clone(),
            ..Default::default()
        }).unwrap_or_else(|e| {
            panic!("{}", e);
        });

        let buffer = Arc::new(Mutex::new(vec![0u32; width * height]));

        draw_loop(60, || {
            if window.is_open() && !window.is_key_down(Key::Escape) {
                let buff = &mut buffer.lock().unwrap();
                for i in buff.iter_mut() {
                    *i = unsafe { get_color((Scrn::PALETTE[0] as u16) | ((Scrn::PALETTE[1] as u16) << 8)) };
                }

                unsafe {
                    for y in 0..7 {
                        let word = Scrn::VRAM[8 + y];
                        for x in 0...7 {
                            let col1 = (((word & 0xFF) >> x) as u8) & 0b1;
                            let col1 = col1 as usize;
                            buff[x + y * width] = get_color((Scrn::PALETTE[col1 * 2] as u16) | ((Scrn::PALETTE[col1 * 2 + 1] as u16) << 8));
                        }
                    }
                }

                window.update_with_buffer(&buff);

                State::Continue
            } else {
                unsafe { Scrn::RUNNING = false; }

                State::Stop
            }
        });

        Screen {
            width: width,
            height: height,
        }
    }
    pub fn new(title: String, width: usize, height: usize) -> Screen {
        Screen::new_scaled(title, width, height, Scale::X2)
    }
}