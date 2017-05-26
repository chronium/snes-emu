use std::sync::atomic::{AtomicU8};
use std::sync::{Mutex, Arc};
use std::cell::{Cell, UnsafeCell};
use std::thread;

use minifb::{Key, Window, WindowOptions, Scale};

pub mod Scrn {
    pub static mut SCREEN: Option<super::Screen> = None;
}

#[derive(Clone)]
pub struct Screen {
    width: usize,
    height: usize,
    pub pal_ind: Arc<Mutex<u8>>,
    pub palette: Arc<Mutex<[u8; 1024]>>,
    memory: Arc<Mutex<Vec<u32>>>,
}

#[inline]
pub fn get_color(color: u16) -> u32 {
    let r = color & 0b000000000011111;
    let g = color & 0b000001111100000;
    let b = color & 0b111110000000000;
    let R = r + r / 32;
    let G = g + g / 32;
    let B = b + b / 32;

    ((b as u32) << 16) | ((g as u32) << 8) | ((r as u32) << 0)
}

impl Screen {
    pub fn new_scaled(title: String, width: usize, height: usize, scale: Scale) -> Screen {
        let mut buff = Arc::new(Mutex::new(vec![0u32; width * height]));
        let mut buff_win = buff.clone();
        let mut palette = Arc::new(Mutex::new([0u8; 1024]));
        let mut palette_win = palette.clone();
        
        thread::spawn(move || {
            let mut window = Window::new(&title, width, height, WindowOptions {
                scale: scale.clone(),
                ..Default::default()
            }).unwrap_or_else(|e| {
                panic!("{}", e);
            });

            while window.is_open() && !window.is_key_down(Key::Escape) {
                let buff = &mut buff_win.lock().unwrap();
                let palette = &palette_win.lock().unwrap();

                for i in buff.iter_mut() {
                    *i = get_color((palette[0] as u16) | (palette[1] as u16));
                }

                window.update_with_buffer(buff);
            }
        });

        Screen {
            width: width,
            height: height,
            pal_ind: Arc::new(Mutex::new(0u8)),
            palette: palette,
            memory: buff,
        }
    }
    pub fn new(title: String, width: usize, height: usize) -> Screen {
        Screen::new_scaled(title, width, height, Scale::X2)
    }
}