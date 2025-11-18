use rand::Rng;

use crate::constants::*;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Chip8State<'a> {
    pub video: &'a [[u8; CHIP8_VIDEO_WIDTH]; CHIP8_VIDEO_HEIGHT],
    pub video_draw: bool,
}

pub struct Chip8 {
    video: [[u8; CHIP8_VIDEO_WIDTH]; CHIP8_VIDEO_HEIGHT], // VRAM
    video_draw: bool,                                     // Redraw frame
    memory: [u8; CHIP8_MEMORY],                           // RAM
    stack: [u16; STACK_HEIGHT],                           // Stack
    v: [u8; REGISTERS_V],                                 // General purpose registers
    i: u16,                                               // I register (store memory addresses)
    pc: u16,                     // Program Counter (store currently executing address)
    sp: u8,                      // Stack Pointer (store topmost level of stack)
    dt: u8,                      // Delay Timer
    st: u8,                      // Sound Timer
    keypad: [bool; KEYPAD_SIZE], // Keypad (16 buttons true or false)
}

impl Chip8 {
    pub fn new() -> Result<Chip8, String> {
        let mut chip8 = Chip8 {
            video: [[0; CHIP8_VIDEO_WIDTH]; CHIP8_VIDEO_HEIGHT],
            video_draw: false,
            memory: [0; CHIP8_MEMORY],
            stack: [0; STACK_HEIGHT],
            v: [0; REGISTERS_V],
            i: 0,
            pc: 0x200, // Programs start at memory location 0x200
            sp: 0,
            dt: 0,
            st: 0,
            keypad: [false; KEYPAD_SIZE],
        };

        for i in 0..FONTSET.len() {
            chip8.memory[i] = FONTSET[i];
        }

        Ok(chip8)
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        self.keypad[key as usize] = pressed;
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        if rom.len() > CHIP8_MEMORY - 0x200 {
            return Err("ROM too large".to_string());
        }
        for (i, &byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
        Ok(())
    }

    pub fn cycle(&mut self) -> Chip8State {
        let opcode = self.gen_opcode();
        self.run_opcode(opcode);

        let should_draw = self.video_draw;
        self.video_draw = false;

        Chip8State {
            video: &self.video,
            video_draw: should_draw,
        }
    }

    pub fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    fn gen_opcode(&mut self) -> u16 {
        if self.pc as usize + 1 >= CHIP8_MEMORY {
            return 0; // Prevent overflow
        }
        let high_byte = self.memory[self.pc as usize];
        let low_byte = self.memory[(self.pc + 1) as usize];
        let opcode = ((high_byte as usize) << 8) | (low_byte as usize);
        return opcode as u16;
    }

    fn run_opcode(&mut self, opcode: u16) {
        let bytes = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        match bytes {
            (0x00, 0x00, 0x00, 0x00) => self.op_0nnn(),
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(opcode),
            (0x03, _, _, _) => self.op_3xkk(opcode),
            (0x04, _, _, _) => self.op_4xkk(opcode),
            (0x05, _, _, 0x00) => self.op_5xy0(opcode),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(opcode),
            (0x08, _, _, 0x01) => self.op_8xy1(opcode),
            (0x08, _, _, 0x02) => self.op_8xy2(opcode),
            (0x08, _, _, 0x03) => self.op_8xy3(opcode),
            (0x08, _, _, 0x04) => self.op_8xy4(opcode),
            (0x08, _, _, 0x05) => self.op_8xy5(opcode),
            (0x08, _, _, 0x06) => self.op_8xy6(opcode),
            (0x08, _, _, 0x07) => self.op_8xy7(opcode),
            (0x08, _, _, 0x0e) => self.op_8xye(opcode),
            (0x09, _, _, 0x00) => self.op_9xy0(opcode),
            (0x0a, _, _, _) => self.op_annn(opcode),
            (0x0b, _, _, _) => self.op_bnnn(opcode),
            (0x0c, _, _, _) => self.op_cxkk(opcode),
            (0x0d, _, _, _) => self.op_dxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(opcode),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(opcode),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(opcode),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(opcode),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(opcode),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(opcode),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(opcode),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(opcode),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(opcode),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(opcode),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(opcode),
            _ => self.pc += 2,
        };
    }

    /// 0nnn - SYS addr
    /// Jump to a machine code routine at nnn (ignored by modern interpreters).
    fn op_0nnn(&mut self) {
        self.pc += 2;
    }

    /// 00E0 - CLS
    /// Clear the display.
    fn op_00e0(&mut self) {
        for x in 0..CHIP8_VIDEO_WIDTH {
            for y in 0..CHIP8_VIDEO_HEIGHT {
                self.video[y][x] = 0;
            }
        }
        self.video_draw = true;
        self.pc += 2; // Next instrction
    }

    /// 00EE - RET
    /// Return from a subroutine.
    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        if self.sp > 0 {
            self.sp -= 1;
        }
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    fn op_2nnn(&mut self, opcode: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc + 2;
        self.pc = opcode & 0x0FFF;
    }

    /// 3xkk - SE Vx, byte
    /// Skip next instruction if Vx == kk.
    fn op_3xkk(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let kk: u8 = (opcode & 0x00FF) as u8;

        if self.v[x] == kk {
            self.pc += 4; // Skip
        } else {
            self.pc += 2; // Next
        }
    }

    /// 4xkk - SNE Vx, byte
    /// Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let kk: u8 = (opcode & 0x00FF) as u8;

        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// 5xy0 - SE Vx, Vy
    /// Skip next instruction if Vx == Vy.
    fn op_5xy0(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// 6xkk - LD Vx, byte
    /// Set Vx = kk.
    fn op_6xkk(&mut self, opcode: u16) {
        self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    /// 7xkk - ADD Vx, byte
    /// Set Vx = Vx + kk.
    fn op_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = self.v[x].wrapping_add(kk);
        self.pc += 2;
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    fn op_8xy0(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        self.v[x] = self.v[y];
        self.pc += 2;
    }

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;
        
        let vx = self.v[x];
        let vy = self.v[y];

        self.v[x] = vx | vy;
        self.v[0xF] = 0;
        self.pc += 2;
    }

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];
        
        self.v[x] = vx & vy;
        self.v[0xF] = 0;
        self.pc += 2;
    }

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];
        
        self.v[x] = vx ^ vy;
        self.v[0xF] = 0;
        self.pc += 2;
    }

    /// 8xy4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];

        let sum = vx as u16 + vy as u16;

        self.v[x] = sum as u8;
        self.v[0xF] = if sum > 255 { 1 } else { 0 };

        self.pc += 2;
    }

    /// 8xy5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];

        self.v[x] = vx.wrapping_sub(vy);
        self.v[0xF] = if vx >= vy { 1 } else { 0 };

        self.pc += 2;
    }

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx >> 1.
    fn op_8xy6(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];
        
        self.v[x] = vy;
        self.v[x] = vx >> 1;
        self.v[0xF] = vx & 0x1;
        self.pc += 2;
    }

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let vx = self.v[x];
        let vy = self.v[y];

        self.v[x] = vy.wrapping_sub(vx);
        self.v[0xF] = if vy >= vx { 1 } else { 0 };

        self.pc += 2;
    }

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx << 1.
    fn op_8xye(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;
        
        let vx = self.v[x];
        let vy = self.v[y];
        
        self.v[x] = vy;
        self.v[x] = vx << 1;
        self.v[0xF] = (vx & 0x80) >> 7;
        self.pc += 2;
    }

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Annn - LD I, addr
    /// Set I = nnn.
    fn op_annn(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
        self.pc += 2;
    }

    /// Bnnn - JP V0, addr
    /// Jump to location nnn + V0.
    fn op_bnnn(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
    }

    /// Cxkk - RND Vx, byte
    /// Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let kk: u8 = (opcode & 0x00FF) as u8;

        let mut rng = rand::rng();
        let random_byte: u8 = rng.random::<u8>();

        self.v[x] = random_byte & kk;
        self.pc += 2;
    }

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, opcode: u16) {
        let n: usize = (opcode & 0x000F) as usize;
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;

        let start_x = self.v[x] as usize;
        let start_y = self.v[y] as usize;

        self.v[0xF] = 0;

        for y_offset in 0..n {
            let sprite_byte = self.memory[(self.i as usize) + y_offset];
            let current_y = (start_y + y_offset) % CHIP8_VIDEO_HEIGHT;

            for x_offset in 0..8 {
                let current_x = (start_x + x_offset) % CHIP8_VIDEO_WIDTH;

                if (sprite_byte & (0x80 >> x_offset)) != 0 {
                    if self.video[current_y][current_x] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.video[current_y][current_x] ^= 1;
                }
            }
        }
        self.video_draw = true;
        self.pc += 2;
    }

    /// Ex9E - SKP Vx
    /// Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;

        if self.keypad[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// ExA1 - SKNP Vx
    /// Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;

        if !self.keypad[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Fx07 - LD Vx, DT
    /// Set Vx = delay timer value.
    fn op_fx07(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.dt;
        self.pc += 2;
    }

    /// Fx0A - LD Vx, K
    /// Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;

        for (key, &pressed) in self.keypad.iter().enumerate() {
            if pressed {
                self.v[x] = key as u8;
                self.pc += 2;
                return;
            }
        }
    }

    /// Fx15 - LD DT, Vx
    /// Set delay timer = Vx.
    fn op_fx15(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        self.dt = self.v[x];
        self.pc += 2;
    }

    /// Fx18 - LD ST, Vx
    /// Set sound timer = Vx.
    fn op_fx18(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        self.st = self.v[x];
        self.pc += 2;
    }

    /// Fx1E - ADD I, Vx
    /// Set I = I + Vx.
    fn op_fx1e(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        self.i = self.i.wrapping_add(self.v[x] as u16);
        self.pc += 2;
    }

    /// Fx29 - LD F, Vx
    /// Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let digit = self.v[x] as u16;
        self.i = digit * 5; // Sprite 5 bytes
        self.pc += 2;
    }

    /// Fx33 - LD B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let value = self.v[x];

        self.memory[self.i as usize] = value / 100;
        self.memory[(self.i + 1) as usize] = (value % 100) / 10;
        self.memory[(self.i + 2) as usize] = value % 10;

        self.pc += 2;
    }

    /// Fx55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;

        for register_index in 0..=x {
            self.memory[(self.i as usize) + register_index] = self.v[register_index];
        }

        self.i += (x as u16) + 1;
        
        self.pc += 2;
    }

    /// Fx65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, opcode: u16) {
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;

        for register_index in 0..=x {
            self.v[register_index] = self.memory[(self.i as usize) + register_index];
        }
        
        self.i += (x as u16) + 1;

        self.pc += 2;
    }
}
