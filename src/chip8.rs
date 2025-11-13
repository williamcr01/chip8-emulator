use crate::constants::*;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Chip8_State<'a> {
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

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        if rom.len() > CHIP8_MEMORY - 0x200 {
            return Err("ROM too large".to_string());
        }
        for (i, &byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
        Ok(())
    }

    pub fn cycle(&mut self) -> Chip8_State{
        let opcode = self.gen_opcode();
        self.run_opcode(opcode);
        
        Chip8_State {
            video: &self.video,
            video_draw: self.video_draw,
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
            (0x00, 0x00, 0x0e, 0x00) => self.op_00E0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00EE(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(),
            (0x03, _, _, _) => self.op_3xkk(),
            (0x04, _, _, _) => self.op_4xkk(),
            (0x05, _, _, 0x00) => self.op_5xy0(),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(),
            (0x08, _, _, 0x01) => self.op_8xy1(),
            (0x08, _, _, 0x02) => self.op_8xy2(),
            (0x08, _, _, 0x03) => self.op_8xy3(),
            (0x08, _, _, 0x04) => self.op_8xy4(),
            (0x08, _, _, 0x05) => self.op_8xy5(),
            (0x08, _, _, 0x06) => self.op_8xy6(),
            (0x08, _, _, 0x07) => self.op_8xy7(),
            (0x08, _, _, 0x0e) => self.op_8xyE(),
            (0x09, _, _, 0x00) => self.op_9xy0(),
            (0x0a, _, _, _) => self.op_Annn(opcode),
            (0x0b, _, _, _) => self.op_Bnnn(),
            (0x0c, _, _, _) => self.op_Cxkk(),
            (0x0d, _, _, _) => self.op_Dxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => self.op_Ex9E(),
            (0x0e, _, 0x0a, 0x01) => self.op_ExA1(),
            (0x0f, _, 0x00, 0x07) => self.op_Fx07(),
            (0x0f, _, 0x00, 0x0a) => self.op_Fx0A(),
            (0x0f, _, 0x01, 0x05) => self.op_Fx15(),
            (0x0f, _, 0x01, 0x08) => self.op_Fx18(),
            (0x0f, _, 0x01, 0x0e) => self.op_Fx1E(),
            (0x0f, _, 0x02, 0x09) => self.op_Fx29(),
            (0x0f, _, 0x03, 0x03) => self.op_Fx33(),
            (0x0f, _, 0x05, 0x05) => self.op_Fx55(),
            (0x0f, _, 0x06, 0x05) => self.op_Fx65(),
            _ => self.pc += 2,
        };
    }

    /// 0nnn - SYS addr
    /// Jump to a machine code routine at nnn (ignored by modern interpreters).
    fn op_0nnn(&mut self) {}

    /// 00E0 - CLS
    /// Clear the display.
    fn op_00E0(&mut self) {
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
    fn op_00EE(&mut self) {}

    /// 1nnn - JP addr
    /// Jump to location nnn.
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    fn op_2nnn(&mut self) {}

    /// 3xkk - SE Vx, byte
    /// Skip next instruction if Vx == kk.
    fn op_3xkk(&mut self) {}

    /// 4xkk - SNE Vx, byte
    /// Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self) {}

    /// 5xy0 - SE Vx, Vy
    /// Skip next instruction if Vx == Vy.
    fn op_5xy0(&mut self) {}

    /// 6xkk - LD Vx, byte
    /// Set Vx = kk.
    fn op_6xkk(&mut self, opcode: u16) {
        self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    /// 7xkk - ADD Vx, byte
    /// Set Vx = Vx + kk.
    fn op_7xkk(&mut self, opcode: u16) {
        self.v[((opcode & 0x0F00) >> 8) as usize] += (opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    fn op_8xy0(&mut self) {}

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self) {}

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self) {}

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self) {}

    /// 8xy4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self) {}

    /// 8xy5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self) {}

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx >> 1.
    fn op_8xy6(&mut self) {}

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self) {}

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx << 1.
    fn op_8xyE(&mut self) {}

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self) {}

    /// Annn - LD I, addr
    /// Set I = nnn.
    fn op_Annn(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
        self.pc += 2;
    }

    /// Bnnn - JP V0, addr
    /// Jump to location nnn + V0.
    fn op_Bnnn(&mut self) {}

    /// Cxkk - RND Vx, byte
    /// Set Vx = random byte AND kk.
    fn op_Cxkk(&mut self) {}

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite at (Vx, Vy), set VF = collision.
    fn op_Dxyn(&mut self, opcode: u16) {
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
    fn op_Ex9E(&mut self) {}

    /// ExA1 - SKNP Vx
    /// Skip next instruction if key with the value of Vx is not pressed.
    fn op_ExA1(&mut self) {}

    /// Fx07 - LD Vx, DT
    /// Set Vx = delay timer value.
    fn op_Fx07(&mut self) {}

    /// Fx0A - LD Vx, K
    /// Wait for a key press, store the value of the key in Vx.
    fn op_Fx0A(&mut self) {}

    /// Fx15 - LD DT, Vx
    /// Set delay timer = Vx.
    fn op_Fx15(&mut self) {}

    /// Fx18 - LD ST, Vx
    /// Set sound timer = Vx.
    fn op_Fx18(&mut self) {}

    /// Fx1E - ADD I, Vx
    /// Set I = I + Vx.
    fn op_Fx1E(&mut self) {}

    /// Fx29 - LD F, Vx
    /// Set I = location of sprite for digit Vx.
    fn op_Fx29(&mut self) {}

    /// Fx33 - LD B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_Fx33(&mut self) {}

    /// Fx55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    fn op_Fx55(&mut self) {}

    /// Fx65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I.
    fn op_Fx65(&mut self) {}
}
