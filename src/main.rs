extern crate rand;
extern crate sdl2;

mod chip8;
mod constants;
mod display_driver;
mod keyboard_driver;

pub use constants::*;

use chip8::Chip8;
use display_driver::DisplayDriver;

use std::env;
use std::fs::File;
use std::io::Read;

struct Rom {
    rom: [u8; 3584], // 4096 - 512 = 3584 (512 bytes reserved for interpreter)
    size: usize,
}

impl Rom {
    fn new(filename: &str) -> Result<Rom, String> {
        let mut f = File::open(filename).map_err(|e| e.to_string())?;
        let mut buffer = [0u8; 3584];

        let rom_size = f.read(&mut buffer).map_err(|e| e.to_string())?;
        println!("Loaded ROM of size: {} bytes", rom_size);

        Ok(Rom {
            rom: buffer,
            size: rom_size,
        })
    }
}

fn main() {
    println!("Welcome, CHIP-8 Emulator starting...");

    let args: Vec<String> = env::args().collect();
    let rom_name = &args[1];
    let sdl2_context = sdl2::init().expect("Failed to initialize SDL2");

    let rom = Rom::new(rom_name).expect("Failed to load ROM");
    let mut display_driver =
        DisplayDriver::new(&sdl2_context).expect("Failed to initialize display driver");
    let mut cpu = Chip8::new().expect("Failed to initialize CHIP-8");

    cpu.load_rom(&rom.rom).expect("Could not load rom");

    let mut event_pump = sdl2_context.event_pump().unwrap();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => return,

                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keyboard_driver::KeyboardDriver::to_chip8_key(keycode) {
                        cpu.set_key(key as u8, true);
                    }
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keyboard_driver::KeyboardDriver::to_chip8_key(keycode) {
                        cpu.set_key(key as u8, false);
                    }
                }
                _ => {}
            }
        }

        let state = cpu.cycle();
        //println!("Drew: {}", state.video_draw);

        if state.video_draw {
            display_driver.draw_screen(state.video);
        }

        // Simple delay to control speed
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
