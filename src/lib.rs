#![allow(unused_variables, dead_code)]
use rand::thread_rng;
use rand::Rng;

///This holds all of the constants (written in capital letters in the code)
mod constants;
use constants::*;

#[derive(Default)]
struct Quirks {
    shift_quirk: bool,
    memory_increment_by_x: bool,
    memory_leave_iunchanged: bool,
    wrap: bool,
    jump: bool,
    vblank: bool,
    logic: bool,
}

///The ram of the chip8 cpu, uses big endian, and is laid out in the following way:
///0x000 start of chip-8 ram
///0x000 to 0x080 reserved for fontset
///0x200 start of most chip-8 programs
///0x600 start of eti 660 chip8 programs
///0xfff end of chip8 ram
#[derive(Debug, Copy, Clone)]
struct Ram {
    bytes: [u8; RAM_SIZE],
}
impl Ram {
    /// Returns the ram with the fontset already loaded
    fn with_fonts() -> Self {
        let mut ram = Self {
            bytes: [0; RAM_SIZE],
        };
        // The fontset
        // this is basically a collection of bytes that make up numbers when in binary
        // to understand them, write them out in binary and put each value below the previous one
        // here are the first bytes F0 90 90 90 f0, placing bytes below one another looks like
        //
        // 1111
        // 1001
        // 1001
        // 1001
        // 1111
        //
        // To make that somewhat clearer to read, lets omit the ones:
        //
        // 1111
        // 1  1
        // 1  1
        // 1  1
        // 1111
        //
        // The zeros are "off" and the ones are "on", this makes the number zero.
        // Guess what the second row of bytes represents:
        //
        //   1
        //  11
        //   1
        //   1
        //  111
        //
        //  its a one.. This is how the fonts in the chip8 are stored
        let fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, //0
            0x20, 0x60, 0x20, 0x20, 0x70, //1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
            0x90, 0x90, 0xF0, 0x10, 0x10, //4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
            0xF0, 0x10, 0x20, 0x40, 0x40, //7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
            0xF0, 0x90, 0xF0, 0x90, 0x90, //a
            0xE0, 0x90, 0xE0, 0x90, 0xE0, //b
            0xF0, 0x80, 0x80, 0x80, 0xF0, //c
            0xE0, 0x90, 0x90, 0x90, 0xE0, //d
            0xF0, 0x80, 0xF0, 0x80, 0xF0, //e
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ]; //f

        for (idx, value) in ram.bytes[0..fontset.len()].iter_mut().enumerate() {
            *value = fontset[idx];
        }
        ram
    }

    ///returns a value from ram
    fn get(self, index: u16) -> u16 {
        ((self.bytes[index as usize] as u16) << 8) | self.bytes[(index + 1) as usize] as u16
    }
}

/// Holds the data from a chip8 file as a vec of bytes
struct RomBuffer {
    buffer: Vec<u8>,
}

impl RomBuffer {
    fn new(file: &str) -> Self {
        let buffer: Vec<u8> = std::fs::read(file).unwrap();
        RomBuffer { buffer }
    }
}
#[derive(Clone, Copy)]
///# Holds all the registers and the sound and delay timers
struct Registers {
    register: [u8; 16],
    vindex: u16,
    /// 0 by default, unless its set to a number then it will just start decrementing by one 60 times per
    /// second
    delay_timer: u8,
    /// Also 0, and decremented with 60hz when set to a number like the delay timer. Except the
    /// sound timer causes a beep when its not zero. So: quiet when 0, beeping when not 0
    sound_timer: u8,
}

impl Registers {
    fn new() -> Self {
        Registers {
            register: [0u8; 16],
            vindex: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
    fn set_index_register(&mut self, value: u16) {
        self.vindex = value;
    }
    fn get_index_register(&self) -> u16 {
        self.vindex
    }
    fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }
    fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }
    fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }
    fn decrement_sound_timer(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    fn decrement_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn get_register(&self, register: u8) -> u8 {
        self.register[register as usize]
    }
    fn set_register(&mut self, register: u8, value: u8) {
        self.register[register as usize] = value;
    }
}
/// 16 16-bit addresses, used to call subroutines or functions and return from them
/// can go into 16 nested subroutines before stack overflows
#[derive(Clone, Copy)]
struct Stack {
    values: [u16; 16],
}
impl Stack {
    fn new() -> Self {
        Stack { values: [0; 16] }
    }
}

struct Cpu {
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,
    /// A list of "buttons", for the keyboard. set to true when pressed, false otherwise
    keyboard: [bool; 16],
    memory: Ram,
    quirks: Quirks,
    registers: Registers,
    stack: Stack,
    /// Only contains indexes to locations in the stack, so 0 through 15
    stackpointer: u8,
}

impl Cpu {
    /// Returns two bytes from memory at the location where the program counter currently points to
    fn fetch(&self) -> u16 {
        //, ram: &Ram) -> u16 {
        self.memory.get(self.program_counter)
    }
    /// Takes two bytes, and decodes what instruction they represent
    fn decode(&self, opcode: u16) -> Instruction {
        match self.first_nibble(opcode) {
            0x0 => match self.last_byte(opcode) {
                0xE0 => Instruction::ClearScreen,
                0xEE => Instruction::ReturnFromSubroutine,
                _ => Instruction::Noop, //panic!("Unimplemented opcode: {:#04x}", opcode),
            },
            0x1 => Instruction::Jump {
                nnn: self.oxxx(opcode),
            },
            0x2 => Instruction::CallSubroutineAtNNN {
                nnn: self.oxxx(opcode),
            },
            0x3 => Instruction::SkipNextInstructionIfXIsKK {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0x4 => Instruction::SkipNextInstructionIfXIsNotKK {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0x5 => Instruction::SkipNextInstructionIfXIsY {
                x: self.second_nibble(opcode),
                y: self.third_nibble(opcode),
            },
            0x6 => Instruction::LoadRegisterX {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0x7 => Instruction::AddToRegisterX {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0x8 => match self.fourth_nibble(opcode) {
                0x0 => Instruction::LoadRegisterXIntoY {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },
                0x1 => Instruction::LoadXOrYinX {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },
                0x2 => Instruction::LoadXAndYInX {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },
                0x3 => Instruction::LoadXXorYInX {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },

                0x4 => Instruction::AddYToX {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },
                0x5 => Instruction::SubYFromX {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },
                0x6 => Instruction::ShiftXRight1 {
                    x: self.second_nibble(opcode),
                },
                0x7 => Instruction::SubXFromY {
                    x: self.second_nibble(opcode),
                    y: self.third_nibble(opcode),
                },

                0xE => Instruction::ShiftXLeft1 {
                    x: self.second_nibble(opcode),
                },
                _ => {
                    panic!("some other 8xxx thingy")
                }
            },
            0x9 => Instruction::SkipNextInstructionIfXIsNotY {
                x: self.second_nibble(opcode),
                y: self.third_nibble(opcode),
            },
            0xA => Instruction::SetIndexRegister {
                nnn: self.oxxx(opcode),
            },
            0xB => Instruction::JumpToAddressPlusV0 {
                nnn: self.oxxx(opcode),
            },
            0xC => Instruction::SetXToRandom {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0xD => Instruction::Display {
                x: self.second_nibble(opcode),
                y: self.third_nibble(opcode),
                n: self.fourth_nibble(opcode),
            },
            0xE => match self.last_byte(opcode) {
                0xA1 => Instruction::SkipIfVxNotPressed {
                    x: self.second_nibble(opcode),
                },
                0x9E => Instruction::SkipIfVxPressed {
                    x: self.second_nibble(opcode),
                },
                _ => {
                    panic!("unimplemented opcode: 0x{:04x}", opcode);
                }
            },
            0xF => match self.last_byte(opcode) {
                0x0A => Instruction::WaitForKeyPressed {
                    x: self.second_nibble(opcode),
                },
                0x07 => Instruction::SetXToDelayTimer {
                    x: self.second_nibble(opcode),
                },
                0x15 => Instruction::SetDelayTimerToX {
                    x: self.second_nibble(opcode),
                },
                0x18 => Instruction::SetSoundTimerToX {
                    x: self.second_nibble(opcode),
                },
                0x1E => Instruction::AddXtoI {
                    x: self.second_nibble(opcode),
                },
                0x29 => Instruction::SetIToSpriteX {
                    x: self.second_nibble(opcode),
                },
                0x33 => Instruction::LoadBCDOfX {
                    x: self.second_nibble(opcode),
                },
                0x55 => Instruction::Write0ThroughX {
                    x: self.second_nibble(opcode),
                },
                0x65 => Instruction::Load0ThroughX {
                    x: self.second_nibble(opcode),
                },
                _ => {
                    panic!("unimplemented opcode: 0x{:06x}", opcode);
                }
            },
            _ => {
                panic!("cannot decode,opcode not implemented: 0x{:04x}", opcode)
            }
        }
    }

    ///Execute the instruction, for details on the instruction, check the instruction enum
    ///definition
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Noop => {
                //do nothing...
            }
            //00E0
            Instruction::ClearScreen => {
                self.display
                    .iter_mut()
                    .for_each(|x| *x = [false; DISPLAY_WIDTH]);
            }
            //00EE
            Instruction::ReturnFromSubroutine => {
                self.stackpointer -= 1;
                self.program_counter = self.stack.values[self.stackpointer as usize];
            }
            //1NNN
            Instruction::Jump { nnn } => {
                self.program_counter = nnn;
            }
            //2NNN
            Instruction::CallSubroutineAtNNN { nnn } => {
                self.stack.values[self.stackpointer as usize] = self.program_counter;
                self.stackpointer += 1;
                self.program_counter = nnn;
            }
            //3XKK
            Instruction::SkipNextInstructionIfXIsKK { x, kk } => {
                let vx = self.registers.get_register(x);
                if vx == kk {
                    self.program_counter += 2;
                }
            }
            //4XKK
            Instruction::SkipNextInstructionIfXIsNotKK { x, kk } => {
                let vx = self.registers.get_register(x);

                if vx != kk {
                    self.program_counter += 2;
                }
            }
            //5XY0
            Instruction::SkipNextInstructionIfXIsY { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);

                if vx == vy {
                    self.program_counter += 2;
                }
            }
            //6XKK
            Instruction::LoadRegisterX { x, kk } => {
                self.registers.set_register(x, kk);
            }
            //7XKK
            Instruction::AddToRegisterX { x, kk } => {
                let vx = self.registers.get_register(x);

                let (tmp, _overflow) = vx.overflowing_add(kk); // as u16 + kk as u16;
                self.registers.set_register(x, tmp);
            }
            //8xy0
            Instruction::LoadRegisterXIntoY { x, y } => {
                let vy = self.registers.get_register(y);
                self.registers.set_register(x, vy);
            }
            //8xy1
            Instruction::LoadXOrYinX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                self.registers.set_register(x, vx | vy);
            }
            //8xy2
            Instruction::LoadXAndYInX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);

                self.registers.set_register(x, vx & vy);
            }
            //8xy3
            Instruction::LoadXXorYInX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                self.registers.set_register(x, vx ^ vy);
            }
            //8xy4
            Instruction::AddYToX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);

                let (res, fv) = vy.overflowing_add(vx);
                self.registers.set_register(x, res);
                self.registers.set_register(0xf, if fv { 1 } else { 0 });
            }

            //8xy5
            Instruction::SubYFromX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);

                let (res, fv) = vx.overflowing_sub(vy);
                self.registers.set_register(x, res);
                self.registers.set_register(0xf, if fv { 0 } else { 1 });
            }

            //8xy6
            Instruction::ShiftXRight1 { x } => {
                let vx = self.registers.get_register(x);
                let vf = if vx & 1 == 1 { 1 } else { 0 };

                self.registers.set_register(x, vx.overflowing_shr(1).0);
                self.registers.set_register(0xF, vf);
            }

            //8xyE
            Instruction::ShiftXLeft1 { x } => {
                let vx = self.registers.get_register(x);
                let fv = (vx as u16 >> 7) & 1;
                let res = self.registers.get_register(x).wrapping_shl(1);

                self.registers.set_register(x, res);
                self.registers
                    .set_register(0xf, if fv == 1 { 1 } else { 0 });
            }
            //8xy7
            Instruction::SubXFromY { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                let (res, fv) = vy.overflowing_sub(vx);
                self.registers.set_register(x, res);
                self.registers.set_register(0xf, if fv { 0 } else { 1 });
            }

            //9XY0
            Instruction::SkipNextInstructionIfXIsNotY { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                if vx != vy {
                    self.program_counter += 2;
                }
            }
            //ANNN
            Instruction::SetIndexRegister { nnn } => {
                self.registers.set_index_register(nnn);
            }
            //BNNN
            Instruction::JumpToAddressPlusV0 { nnn } => {
                let v0 = (self.registers.get_register(0) & 0xf) as u16;
                self.program_counter = nnn + v0;
            }
            //cxkk
            Instruction::SetXToRandom { x, kk } => {
                let mut rng = thread_rng();
                let random_number = rng.gen_range(0..=255);
                self.registers.set_register(x, random_number & kk);
            }
            //DXYN
            Instruction::Display { x, y, n } => {
                //drawing at (start_x, start_y) on the display, wraps around if out of bounds
                let start_x = (self.registers.get_register(x) % DISPLAY_WIDTH as u8) as usize;
                let start_y = (self.registers.get_register(y) % DISPLAY_HEIGHT as u8) as usize;

                let sprite_start = self.registers.get_index_register() as usize;
                self.registers.set_register(0xF, 0);

                //move over all rows of the sprite (it has n rows)
                for sprite_row in 0..n as usize {
                    if sprite_start + sprite_row >= RAM_SIZE {
                        return;
                    }
                    let sprite = self.memory.bytes[sprite_start + sprite_row];
                    for sprite_column in 0..8usize {
                        let pixel_row = start_x + sprite_column;
                        let pixel_column = start_y + sprite_row;

                        let sprite_pixel_set = sprite >> (7 - sprite_column) & 1 == 1;

                        //check so as to *not* draw out of bounds of the display
                        if pixel_row < DISPLAY_WIDTH && pixel_column < DISPLAY_HEIGHT {
                            if self.display[pixel_column][pixel_row] && sprite_pixel_set {
                                self.registers.set_register(0xf, 1);
                            }
                            self.display[pixel_column][pixel_row] ^= sprite_pixel_set;
                        }
                    }
                }
            }
            //exa1
            Instruction::SkipIfVxNotPressed { x } => {
                //let key = Cpu::u8_to_keycode(self.registers.get_register(x) & 0xf);
                if !self.keyboard[x as usize] {
                    self.program_counter += 2;
                }
                //if !is_key_down(key) {

                //}
            }
            //ex9e
            Instruction::SkipIfVxPressed { x } => {
                //let key = Cpu::u8_to_keycode(self.registers.get_register(x) & 0xf);

                if self.keyboard[x as usize] {
                    self.program_counter += 2;
                }
            }
            //fx0a
            Instruction::WaitForKeyPressed { x } => {
                match self.get_pressed_key() {
                    //Do not advance the program counter, the entire system must wait for a key to be pressed
                    None => self.program_counter -= 2,
                    //Original cosmac vip only registered a kley when it was pressed *and* released
                    Some(x) => {}
                }
            }
            //fx07
            Instruction::SetXToDelayTimer { x } => {
                let vdt = self.registers.get_delay_timer();
                self.registers.set_register(x, vdt);
            }
            //fx15
            Instruction::SetDelayTimerToX { x } => {
                let vx = self.registers.get_register(x);
                self.registers.set_delay_timer(vx);
            }
            Instruction::SetSoundTimerToX { x } => {
                let vx = self.registers.get_register(x);
                self.registers.set_sound_timer(vx);
            }
            //fx1E
            Instruction::AddXtoI { x } => {
                let vx = self.registers.get_register(x) as u16;
                let vi = self.registers.get_index_register();
                let added = vi + vx;

                self.registers.set_index_register(added);
            }
            //fx29
            Instruction::SetIToSpriteX { x } => {
                //let vx = (self.registers.get_register(x) * 5) as u16 & 0xffff;
                let vx = (self.registers.get_register(x) * 5) as u16;
                //the sprite at *index* x, not location x.
                self.registers.set_index_register(vx);
            }
            Instruction::LoadBCDOfX { x } => {
                let vx = self.registers.get_register(x);
                let store_index = self.registers.get_index_register() as usize;
                self.memory.bytes[store_index] = vx / 100;
                self.memory.bytes[store_index + 1] = (vx % 100) / 10;
                self.memory.bytes[store_index + 2] = (vx % 100) % 10;
            }
            //fx55
            Instruction::Write0ThroughX { x } => {
                let vi = self.registers.get_index_register() as usize;

                for register in 0..x + 1 {
                    let register_value = self.registers.get_register(register);
                    self.memory.bytes[vi + register as usize] = register_value;
                }
            }
            //fx65
            Instruction::Load0ThroughX { x } => {
                let vi = self.registers.get_index_register() as usize;
                for i in 0..x + 1 {
                    self.registers
                        .set_register(i, self.memory.bytes[vi + i as usize]);
                }
            }
        }
    }
    /// A nibble is 4 bits, so this returns the first 4 bits of an opcode
    fn first_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 12) & 0xF) as u8
    }
    fn second_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xf) as u8
    }
    fn third_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xf) as u8
    }
    fn fourth_nibble(&self, opcode: u16) -> u8 {
        (opcode as u8) & 0xf
    }
    /// Returns the last full byte byte of an opcode
    fn last_byte(&self, opcode: u16) -> u8 {
        (opcode & 0xff) as u8
    }
    /// Returns the the last 12 bits of an opcode
    fn oxxx(&self, opcode: u16) -> u16 {
        opcode & 0xfff
    }

    fn get_pressed_key(&self) -> Option<usize> {
        self.keyboard
            .iter()
            .position(|button_pressed| *button_pressed)
    }

    fn get_display_contents(&self) -> [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        self.display
    }
    /// A single cpu cycle, fetches, decodes, executes opcodes and
    /// decrements the timers if relevant. also updates the program_counter
    fn cycle(&mut self) {
        let opcode = self.fetch();
        self.program_counter += 2;
        let instruction = self.decode(opcode);
        self.execute(instruction);

        self.registers.decrement_sound_timer();
        self.registers.decrement_delay_timer();
    }

    /// Creates a new cpu object, with the contents of a rom file loaded in to memory
    fn new(rom: RomBuffer) -> Self {
        let display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        let program_counter = 0x200;
        let registers = Registers::new();
        let keyboard = [false; 16];
        let quirks = Quirks::default();
        let mut memory = Ram::with_fonts();
        for (x, y) in rom.buffer.iter().enumerate() {
            memory.bytes[0x200 + x] = *y;
        }

        let stack = Stack::new();

        Self {
            display,
            program_counter,
            registers,
            keyboard,
            quirks,
            memory,
            stack,
            stackpointer: 0,
        }
    }
}

/// A list of every instruction in the chip8 language
/// nnn is a hexadecimal memory address, it's 12 bits long
/// nn is a hexadecimal byte, it's 8 bits
/// n is what's called a "nibble", it's 4 bits
/// X and Y are registers
#[derive(Debug, PartialEq)]
enum Instruction {
    /// The "no-op" instruction, this does absolutely nothing, by design.
    Noop, //0nnn
    /// Turns all the pixels to off (false, in our case)
    ClearScreen, //00e0
    /// Sets the program counter to the last address in the stack
    ReturnFromSubroutine, //00ee
    /// Sets the program counter to whatever nnn is
    Jump {
        nnn: u16,
    }, //1nnn
    CallSubroutineAtNNN {
        nnn: u16,
    }, //2nnn
    /// Set register x to the value kk
    LoadRegisterX {
        x: u8,
        kk: u8,
    }, //6xkk
    /// Adds the value kk to register x
    AddToRegisterX {
        x: u8,
        kk: u8,
    }, //7xnn
    /// Sets the value of register x to the result of binary OR-ing register x and y
    LoadXOrYinX {
        x: u8,
        y: u8,
    }, //8xy1
    /// Sets the value of register x to the result of binary AND-ing register x and y
    LoadXAndYInX {
        x: u8,
        y: u8,
    }, //8xy2
    /// Sets the value of register x to the result of binary XOR-ing register x and y
    LoadXXorYInX {
        x: u8,
        y: u8,
    }, //8xy3
    /// Sets the value of register x to the value of itself added to that of register y
    AddYToX {
        x: u8,
        y: u8,
    }, //8xy4
    /// Sets the value of register x to the value of itself subtracted from that of register y, so
    /// vx - vy
    SubYFromX {
        x: u8,
        y: u8,
    }, //8xy5
    /// shift the value of register x one bit to the right
    ShiftXRight1 {
        x: u8,
    }, //8xy6
    /// shift the value of register x one bit to the left
    ShiftXLeft1 {
        x: u8,
    }, //8xyE
    /// Sets the value of register x to the value of register y subtracted from itself, so vy - vx
    SubXFromY {
        x: u8,
        y: u8,
    }, //8xy7
    LoadRegisterXIntoY {
        x: u8,
        y: u8,
    }, //Stores the value of register Vy in register Vx
    SetIndexRegister {
        nnn: u16,
    }, //ANNN set index register I to nnn
    JumpToAddressPlusV0 {
        nnn: u16,
    }, //BNNN jump to address nnn + v0
    SkipNextInstructionIfXIsKK {
        x: u8,
        kk: u8,
    }, //skips the next instruction only if the register X holds the value kk
    SkipNextInstructionIfXIsNotKK {
        x: u8,
        kk: u8,
    }, //same as previous, except skips if register x does not hold value kk
    SkipNextInstructionIfXIsY {
        x: u8,
        y: u8,
    },
    SkipNextInstructionIfXIsNotY {
        x: u8,
        y: u8,
    },
    SetXToRandom {
        x: u8,
        kk: u8,
    }, //cxkk
    Display {
        x: u8,
        y: u8,
        n: u8,
    }, //DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
    SkipIfVxNotPressed {
        x: u8,
    }, //exa1
    SkipIfVxPressed {
        x: u8,
    }, //ex9e
    WaitForKeyPressed {
        x: u8,
    }, //fx0a
    SetXToDelayTimer {
        x: u8,
    }, //fx07
    SetDelayTimerToX {
        x: u8,
    }, //Fx15
    SetSoundTimerToX {
        x: u8,
    }, //fx18
    AddXtoI {
        x: u8,
    }, //fx1e
    SetIToSpriteX {
        x: u8,
    }, //fx29
    LoadBCDOfX {
        x: u8,
    }, //fx33
    Write0ThroughX {
        x: u8,
    }, //fx55
    Load0ThroughX {
        x: u8,
    }, //fx65
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn it_loads_files() {
        let rom_buffer = RomBuffer::new("tests/1-chip8-logo.8o");
        assert!(rom_buffer.buffer[0] == 0x23);
    }

    #[test]
    fn it_can_initialize() {
        let buffer = RomBuffer::new("tests/1-chip8-logo.8o");
        let instance = Cpu::new(buffer);
        assert!(instance.program_counter == 0x200);
    }

    #[test]
    fn it_can_fetch_instruction() {
        let buffer = RomBuffer::new("tests/1-chip8-logo.8o");
        let instance = Cpu::new(buffer);
        assert!(instance.fetch() == 0x2320);
    }

    // instructions in order of https://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf
    #[test]
    fn executes_00E0() {
        // Clears the display
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x00, 0xE0],
        });
        instance.display[0][0] = true;
        instance.cycle();
        assert!(instance.display[0][0] == false);
    }
    #[test]
    fn executes_00EE() {
        // Return from a subroutine
        // sets the counter to the address at the top of the stack, and subtracts 1 from the stack pointer
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x00, 0xEE],
        });
        instance.stack.values[0] = 0x201;
        instance.stackpointer = 1;
        instance.cycle();
        assert!(instance.stackpointer == 0);
        assert!(instance.program_counter == 0x201);
    }

    #[test]
    fn executes_1NNN() {
        // Jumps to location nnn, this should set the program counter to nnn
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x11, 0x23],
        });
        instance.cycle();
        assert_eq!(instance.program_counter == 0x123, true);
    }

    #[test]
    fn executes_2NNN() {
        // Call subroutine at nnn, this should:
        // 1. increment the stack pointer
        // 2. put the current program counter at the top of the stack
        // 3. sets the program counter to nnn
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x21, 0x23],
        });
        instance.cycle();
        println!("{:#x}", instance.stack.values[0]);
        assert!(instance.stackpointer == 1);
        //I'm not sure why this isn't 0x200
        assert!(instance.stack.values[0] == 0x202);
        assert!(instance.program_counter == 0x123);
    }

    #[test]
    fn executes_3XKK() {
        //Should increment the program counter by two if  register VX is equal to NN
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x31, 0x00],
        });
        instance.cycle();
        assert!(instance.program_counter == 0x204);

        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x31, 0x01],
        });
        instance.cycle();
        assert!(instance.program_counter == 0x202);
    }

    #[test]
    fn executes_4Xkk() {
        //Should increment the program counter by two if  register VX is not equal to NN
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x41, 0x00],
        });
        instance.cycle();
        assert!(instance.program_counter == 0x202);

        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x41, 0x01],
        });
        instance.cycle();
        assert!(instance.program_counter == 0x204);
    }

    #[test]
    fn executes_5XY0() {
        //Should increment the program counter by two if register vs equals register vy
        //here regsiter x and y are both 0, which should update the pc to 0x204
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x51, 0x20],
        });
        instance.cycle();
        assert!(instance.program_counter == 0x204);

        //Here register 1 will be set to 5, so it should leave the pc at 0x202
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x51, 0x20],
        });
        instance.registers.set_register(1, 5);
        instance.cycle();
        assert!(instance.program_counter == 0x202);
    }

    #[test]
    fn executes_6XKK() {
        //Should put the value KK in to register X
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x60, 0x22],
        });
        instance.cycle();
        assert!(instance.registers.get_register(0) == 0x22);
    }

    #[test]
    fn executes_7XKK() {
        //Should put the value KK plus the current value of register x in to register X
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x71, 0x05],
        });
        instance.registers.set_register(0x1, 0x4);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 0x09);
    }

    #[test]
    fn executes_8XY0() {
        //Should store the value of register y in to register x
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x20],
        });
        instance.registers.set_register(0x2, 0x4);
        instance.cycle();
        assert!(instance.registers.get_register(2) == 0x4);
    }

    #[test]
    fn executes_8XY1() {
        //Should store the value of register y ORED with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x21],
        });
        instance.registers.set_register(0x2, 4);
        instance.registers.set_register(0x1, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == (4 | 2));
    }

    #[test]
    fn executes_8XY2() {
        //Should store the value of register y ANDed with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x22],
        });
        instance.registers.set_register(0x2, 4);
        instance.registers.set_register(0x1, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == (4 & 2));
    }

    #[test]
    fn executes_8XY3() {
        //Should store the value of register y XORed with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x23],
        });
        instance.registers.set_register(0x2, 4);
        instance.registers.set_register(0x1, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == (4 ^ 2));
    }

    #[test]
    fn executes_8XY4() {
        //Should store the value of register y ADDED to whatever is in register y in to register x
        //if the value is bigger than 8 bits (i.e 255), register f should be set to 1, 0 otherwise, and only the lowest
        //8 bit of the result should be kept and stored in register x
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x24],
        });
        instance.registers.set_register(0x2, 200);
        instance.registers.set_register(0x1, 1);
        instance.cycle();

        assert!(instance.registers.get_register(1) == (200 + 1));
        assert!(instance.registers.get_register(0xf) == 0);

        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x24],
        });
        instance.registers.set_register(0x2, 200);
        instance.registers.set_register(0x1, 60);
        instance.cycle();
        //This shows an overflow (200 + 60 is bigger than 8 bits), only the last 8 bits should be saved (so AND'ed with 0xff)
        assert!(instance.registers.get_register(1) as u16 == (200 + 60) & 0xff);
        //and the carry flag should be set
        assert!(instance.registers.get_register(0xf) == 1);
    }

    #[test]
    fn executes_8XY5() {
        //Should store the value of register y subtracted from whatever is in register y in to register x
        //if an underflow occurs, register f is set to 0, otherwise its 1. So the opposite of what you'd expect
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x25],
        });
        instance.registers.set_register(0x1, 10);
        instance.registers.set_register(0x2, 5);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 10 - 5);
        assert!(instance.registers.get_register(0xf) == 1);

        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x25],
        });
        instance.registers.set_register(0x1, 5);
        instance.registers.set_register(0x2, 10);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 5u8.overflowing_sub(10).0);
        assert!(instance.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XY6() {
        //Should store the value of register x shifted right by one in register x
        //sets register f to 1 if the least significant bit of vx is 1, otherwise it sets it to 0
        //then vx is divided by 2?
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x26],
        });
        instance.registers.set_register(0x1, 17);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 8);
        //assert!(instance.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XY7() {
        //Should store the value of register x subtracted from the value in register y, inside register x. register f is said when we didn't borrow.
        // again, opposite of what you'd expect.
        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x27],
        });
        instance.registers.set_register(0x1, 2);
        instance.registers.set_register(0x2, 10);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 8);
        assert!(instance.registers.get_register(0xf) == 1);

        let mut instance = Cpu::new(RomBuffer {
            buffer: vec![0x81, 0x27],
        });
        instance.registers.set_register(0x1, 10);
        instance.registers.set_register(0x2, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 248);
        assert!(instance.registers.get_register(0xf) == 0);
    }
}
