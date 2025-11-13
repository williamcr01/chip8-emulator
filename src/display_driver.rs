use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::{CHIP8_VIDEO_HEIGHT, CHIP8_VIDEO_WIDTH, VIDEO_SCALE};

pub struct DisplayDriver {
    canvas: Canvas<Window>,
    scale: u32,
}

impl DisplayDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<DisplayDriver, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "CHIP-8 Emulator",
                (CHIP8_VIDEO_WIDTH * VIDEO_SCALE) as u32,
                (CHIP8_VIDEO_HEIGHT * VIDEO_SCALE) as u32,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok(DisplayDriver {
            canvas,
            scale: VIDEO_SCALE as u32,
        })
    }

    pub fn draw_screen(&mut self, vram: &[[u8; CHIP8_VIDEO_WIDTH]; CHIP8_VIDEO_HEIGHT]) {
        for (y, row) in vram.iter().enumerate() {
            for (x, &pixel_value) in row.iter().enumerate() {
                let x_size = (x * VIDEO_SCALE) as u32;
                let y_size = (y * VIDEO_SCALE) as u32;
                
                if pixel_value == 0 {
                    self.canvas.set_draw_color(Color::BLACK);
                } else {
                    self.canvas.set_draw_color(Color::WHITE);
                }
                let _ = self.canvas.fill_rect(Rect::new(
                    x_size as i32,
                    y_size as i32,
                    VIDEO_SCALE as u32,
                    VIDEO_SCALE as u32,
                ));
            }
        }
        self.present();
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.present();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
