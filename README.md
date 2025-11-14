# CHIP-8 Emulator

A CHIP-8 emulator written in Rust using SDL2 for graphics and input.

![Pong running on the emulator](img/pong.png)

## Building

With rust installed run:

```bash
cargo build --release
```

## Running

To run a CHIP-8 ROM:

```bash
cargo run --release <path-to-rom>
```

Example:
```bash
cargo run --release roms/pong.ch8
```

## Controls

The CHIP-8 uses a 16-key hexadecimal keypad (0-F). The keys are mapped as follows:

```
CHIP-8 Keypad:       Keyboard Mapping:
1 2 3 C              1 2 3 4
4 5 6 D              Q W E R
7 8 9 E              A S D F
A 0 B F              Z X C V
```
