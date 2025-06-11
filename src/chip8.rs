const CHIP8_MEMORY: usize = 4096;
const CHIP8_VIDEO_WIDTH: usize = 64;
const CHIP8_VIDEO_HEIGHT: usize = 32;
const STACK_HEIGHT: usize = 16;
const REGISTERS_V: usize = 16;
const KEYPAD_SIZE: usize = 16;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

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
    fn new(&mut self) -> Result<Chip8, String> {
        let mut chip8 = Chip8 {
            video: [[0; CHIP8_VIDEO_WIDTH]; CHIP8_VIDEO_HEIGHT],
            video_draw: false,
            memory: [0; CHIP8_MEMORY],
            stack: [0; STACK_HEIGHT],
            v: [0; REGISTERS_V],
            i: 0,
            pc: 0,
            sp: 0,
            dt: 0,
            st: 0,
            keypad: [false; KEYPAD_SIZE],
        };

        for i in 0..FONTSET.len() {
            chip8.memory[i] = FONTSET[0];
        }

        Ok(chip8)
    }

    /// 0nnn - SYS addr  
    /// Jump to a machine code routine at nnn (ignored by modern interpreters).
    fn op_0nnn(&mut self) {}

    /// 00E0 - CLS  
    /// Clear the display.
    /// TODO: IBM
    fn op_00E0(&mut self) {
        for x in CHIP8_VIDEO_WIDTH {
            for y in CHIP8_VIDEO_HEIGHT{
                self.video[x][y] = 0;
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
    /// TODO: IBM
    fn op_1nnn(&mut self, opcode: usize) {
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
    /// TODO: IBM
    fn op_6xkk(&mut self, opcode: usize) {
        self.v[(opcode & 0x0F00) >> 8] = opcode & 0x00FF;
        pc += 2;
    }

    /// 7xkk - ADD Vx, byte  
    /// Set Vx = Vx + kk.
    /// TODO: IBM
    fn op_7xkk(&mut self, opcode: usize) {
        self.v[(opcode & 0x0F00) >> 8] += opcode & 0x00FF;
        pc += 2;
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
    /// TODO: IBM
    fn op_Annn(&mut self, opcode: usize) {
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
    /// TODO: IBM
    fn op_Dxyn(&mut self) {}

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
