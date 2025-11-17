pub const CHIP8_MEMORY: usize = 4096;
pub const CHIP8_VIDEO_WIDTH: usize = 64;
pub const CHIP8_VIDEO_HEIGHT: usize = 32;
pub const STACK_HEIGHT: usize = 16;
pub const REGISTERS_V: usize = 16;
pub const KEYPAD_SIZE: usize = 16;

pub const VIDEO_SCALE: usize = 20;

// Timing constants
pub const CPU_HZ: u32 = 500; // CPU cycles per second
pub const TIMER_HZ: u32 = 60; // Timer ticks per second
pub const CYCLES_PER_TIMER_TICK: u32 = CPU_HZ / TIMER_HZ; // ~11-12 cycles per timer tick