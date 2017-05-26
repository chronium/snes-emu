use std::sync::atomic::{AtomicU8};
use std::sync::{Mutex, Arc};
use std::cell::{Cell, UnsafeCell};
use std::thread;

use minifb::{Key, Window, WindowOptions, Scale};

pub mod Scrn {
    pub static mut SCREEN: Option<super::Screen> = None;
    pub static mut PALETTE_INDEX: u16 = 0u16;
    pub static mut PALETTE: [u8; 1024] = [0u8; 1024];
}

#[derive(Clone)]
pub struct Screen {
    width: usize,
    height: usize,
}

#[inline]
pub fn get_color(color: u16) -> u32 {
    let r = color & 0b000000000011111;
    let g = color & 0b000001111100000;
    let b = color & 0b111110000000000;
    let R = (r * 255) / 31;
    let G = (g * 255) / 31;
    let B = (b * 255) / 31;

    ((R as u32) << 16) | ((G as u32) << 8) | ((B as u32) << 0) | 0xFF000000
}

impl Screen {
    pub fn new_scaled(title: String, width: usize, height: usize, scale: Scale) -> Screen {
        thread::spawn(move || {
            let mut window = Window::new(&title, width, height, WindowOptions {
                scale: scale.clone(),
                ..Default::default()
            }).unwrap_or_else(|e| {
                panic!("{}", e);
            });

            let mut buff = vec![0; width * height];

            while window.is_open() && !window.is_key_down(Key::Escape) {
                for i in buff.iter_mut() {
                    *i = unsafe { get_color((Scrn::PALETTE[0] as u16) | (Scrn::PALETTE[1] as u16)) };
                }

                window.update_with_buffer(&buff);
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