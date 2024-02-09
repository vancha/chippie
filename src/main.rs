use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
/*use crossterm::{
   event::{ KeyCode, KeyEventKind},
   terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
   ExecutableCommand,
};*/
use rand::Rng;
use ratatui::{
    prelude::{Buffer, CrosstermBackend, Rect, Terminal},
    widgets::Widget,
};
use std::io::{stdout, Result};

//all of the following dependencies are used for my janky implementation for input
use std::collections::HashMap;
//use std::cell::RefCell;
//use std::rc::Rc;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096; //in bytes :)
const CYCLES_PER_FRAME: usize = 15;

///The ram of the chip8 cpu, uses big endian, and is laid out in the following way:
///0x000 start of chip-8 ram
///0x000 to 0x080 reserved for fontset
///0x200 start of most chip-8 programs
///0x600 start of eti 660 chip8 programs
///0xfff end of chip8 ram
#[derive(Debug, Copy, Clone)]
struct RAM {
    bytes: [u8; RAM_SIZE],
}
impl RAM {
    ///Returns an empty ram object, no fonts, no rom, just empty bytes
    fn new() -> Self {
        Self {
            bytes: [0; RAM_SIZE],
        }
    }

    fn with_fonts() -> Self {
        let mut ram = Self {
            bytes: [0; RAM_SIZE],
        };

        let fontset = vec![
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

struct RomBuffer {
    buffer: Vec<u8>,
}
impl RomBuffer {
    fn new(file: &str) -> Self {
        let buffer: Vec<u8> = std::fs::read(file).unwrap();
        RomBuffer { buffer: buffer }
    }
}
#[derive(Clone, Copy)]
///# All 16 8 bit registers, and the 16 bit I register
struct Registers {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8,

    vindex: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Registers {
    fn new() -> Self {
        Registers {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,

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
        return self.delay_timer;
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
        match register {
            0 => self.v0,
            1 => self.v1,
            2 => self.v2,
            3 => self.v3,
            4 => self.v4,
            5 => self.v5,
            6 => self.v6,
            7 => self.v7,
            8 => self.v8,
            9 => self.v9,
            0xA => self.va,
            0xB => self.vb,
            0xC => self.vc,
            0xD => self.vd,
            0xE => self.ve,
            0xF => self.vf,
            _ => {
                panic!("Invalid register");
            }
        }
    }
    fn set_register(&mut self, register: u8, value: u8) {
        match register {
            0 => {
                self.v0 = value;
            }
            1 => {
                self.v1 = value;
            }
            2 => {
                self.v2 = value;
            }
            3 => {
                self.v3 = value;
            }
            4 => {
                self.v4 = value;
            }
            5 => {
                self.v5 = value;
            }
            6 => {
                self.v6 = value;
            }
            7 => {
                self.v7 = value;
            }
            8 => {
                self.v8 = value;
            }
            9 => {
                self.v9 = value;
            }
            0xA => {
                self.va = value;
            }
            0xB => {
                self.vb = value;
            }
            0xC => {
                self.vc = value;
            }
            0xD => {
                self.vd = value;
            }
            0xE => {
                self.ve = value;
            }
            0xF => {
                self.vf = value;
            }
            _ => {
                panic!("Invalid register");
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Stack {
    values: [u16; 16],
}
impl Stack {
    fn new() -> Self {
        Stack { values: [0; 16] }
    }
}

#[derive(Clone, Copy)]
struct CPU {
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,
    memory: RAM,
    keyboard: [(Key,bool);16],//keyboard scancodes, and their pressed state, true = pressed
    registers: Registers,
    stack: Stack, //stack for keeping track of where to return to after subroutine, can go into 16 nested subroutines before stackoverflow
    stackpointer: u8, //only contains indexes to locations in the stack, so 0 through 15
}

//ratatui's 'widget' trait, to draw the display :)
impl Widget for CPU {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                buf.get_mut(col as u16, row as u16)
                    .set_char(match self.display[row][col] {
                        true => '█',
                        false => ' ',
                    });
            }
        }
    }
}
impl CPU {
    fn fetch(&self, ram: &RAM) -> u16 {
        ram.get(self.program_counter)
    }

    fn decode(&self, opcode: u16) -> Instruction {
        match self.first_nibble(opcode) {
            0x00 => match self.last_byte(opcode) {
                0xE0 => Instruction::ClearScreen,
                0xEE => Instruction::ReturnFromSubroutine,

                _ => Instruction::JUMP { nnn: 0x200 }, //panic!("Unimplemented opcode: {:#06x}", opcode),
            },
            0x1 => Instruction::JUMP {
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
            0xC => Instruction::SetXToRandom {
                x: self.second_nibble(opcode),
                kk: self.last_byte(opcode),
            },
            0xD => Instruction::DISPLAY {
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
                    panic!("unimplemented opcode: 0x{:04x}", opcode);
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
            Instruction::JUMP { nnn } => {
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

                let (tmp, overflow) = vx.overflowing_add(kk); // as u16 + kk as u16;
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
            //cxkk
            Instruction::SetXToRandom { x, kk } => {
                let mut rng = rand::thread_rng();
                let random_number = rng.gen_range(0..=255);
                self.registers.set_register(x, random_number & kk);
            }
            //DXYN
            Instruction::DISPLAY { x, y, n } => {
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
                    for sprite_column in 0..8 as usize {
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
                //todo!("Opcode yet to be implemented");
            }
            Instruction::SkipIfVxPressed { x } => {
                //todo!("Opcode yet to be implemented");
            }
            //fx07
            Instruction::SetXToDelayTimer { x } => {
                let vdt = self.registers.get_delay_timer();
                self.registers.set_register(x, vdt);
            }

            //fx15
            Instruction::SetDelayTimerToX { x } => {
                let vx = self.registers.get_register(x);
                self.registers.delay_timer = vx;
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
                let vx = self.registers.get_register(x);
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
    //returns the first 4 bits of the opcode as a byte
    fn first_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 12) & 0xF) as u8
    }
    //returns the second 4 bits of the opcode as a byte
    fn second_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xf) as u8
    }
    fn third_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xf) as u8
    }
    fn fourth_nibble(&self, opcode: u16) -> u8 {
        (opcode as u8) & 0xf
    }
    //returns the last byte
    fn last_byte(&self, code: u16) -> u8 {
        (code & 0xff) as u8
    }
    fn oxxx(&self, code: u16) -> u16 {
        code & 0xfff
    }

    fn cycle(&mut self) {
        let opcode = self.fetch(&self.memory);

        self.program_counter += 2;

        let instruction = self.decode(opcode);

        self.execute(instruction);

        self.registers.decrement_sound_timer();
        self.registers.decrement_delay_timer();
    }

    fn new(rom: RomBuffer) -> Self {
        let mut memory = RAM::with_fonts();
        for (x, y) in rom.buffer.iter().enumerate() {
            memory.bytes[0x200 + x] = *y;
        }

        Self {
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            program_counter: 0x200,
            registers: Registers::new(),
            memory: memory,
            stack: Stack::new(),
            keyboard: [(Key::Key0,false),(Key::Key1,false),(Key::Key2,false),(Key::Key3,false),(Key::Key4,false),(Key::Key5,false),(Key::Key6,false),(Key::Key7,false),(Key::Key8,false),(Key::Key9,false),(Key::KeyA,false),(Key::KeyB,false),(Key::KeyC,false),(Key::KeyD,false),(Key::KeyE,false),(Key::KeyF,false)],
            stackpointer: 0,
        }
    }
}

///A list of every instruction in the chip8 language
///nnn is a hexadecimal memory address, it's 12 bits long
///nn is a hexadecimal byte, it's 8 bits
///n is what's called a "nibble", it's 4 bits
///X and Y are registers
enum Instruction {
    ClearScreen,          //00e0
    ReturnFromSubroutine, //00ee
    JUMP { nnn: u16 },    //1nnn where nnn is a 12 bit value (lowest 12 bits of the instruction)
    AddToRegisterX { x: u8, kk: u8 },
    CallSubroutineAtNNN { nnn: u16 },
    LoadRegisterX { x: u8, kk: u8 }, //6xkk puts the value kk into Vx
    LoadXOrYinX { x: u8, y: u8 },    //8xy1
    LoadXAndYInX { x: u8, y: u8 },   //8xy2
    LoadXXorYInX { x: u8, y: u8 },   //8xy3
    AddYToX { x: u8, y: u8 },        //8xy4
    SubYFromX { x: u8, y: u8 },      //8xy5
    ShiftXRight1 { x: u8 },          //8xy6
    ShiftXLeft1 { x: u8 },           //8xyE
    SubXFromY { x: u8, y: u8 },      //8xy7
    LoadRegisterXIntoY { x: u8, y: u8 }, //Stores the value of register Vy in register Vx
    SetIndexRegister { nnn: u16 },   //ANNN set index register I to nnn
    SkipNextInstructionIfXIsKK { x: u8, kk: u8 }, //skips the next instruction only if the register X holds the value kk
    SkipNextInstructionIfXIsNotKK { x: u8, kk: u8 }, //same as previous, except skips if register x does not hold value kk
    SkipNextInstructionIfXIsY { x: u8, y: u8 },
    SkipNextInstructionIfXIsNotY { x: u8, y: u8 },
    SetXToRandom { x: u8, kk: u8 },  //cxkk
    DISPLAY { x: u8, y: u8, n: u8 }, //DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
    SkipIfVxNotPressed { x: u8 },    //exa1
    SkipIfVxPressed { x: u8 },       //ex9e
    SetXToDelayTimer { x: u8 },      //fx07
    SetDelayTimerToX { x: u8 },      //Fx15
    SetSoundTimerToX { x: u8 },      //fx18
    AddXtoI { x: u8 },               //fx1e
    SetIToSpriteX { x: u8 },         //fx29
    LoadBCDOfX { x: u8 },            //fx33
    Write0ThroughX { x: u8 },        //fx55
    Load0ThroughX { x: u8 },         //fx65
}

#[derive(Debug,Copy,Clone, Hash, Eq, PartialEq)]
enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}

fn main() -> Result<()> {

    //creating a chip8 cpu object with a rom loaded
    let b = RomBuffer::new("./pong.ch8");
    let mut c = CPU::new(b);

    //folowing code is all for setting up the tui library "ratatui"
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    //ratatui main loop, runs 60 times per second
    loop {
        //the "cyles per frame" metric for out chip8 cpu
        for _ in 0..=CYCLES_PER_FRAME {
            c.cycle();
        }
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(c, area);
        })?;

        //handle input events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if key.kind == KeyEventKind::Press {
                            break;
                        }
                    }
                    _ => {}
                    KeyCode::Char('w') => {
                        if key.kind == KeyEventKind::Press {
                            c.keyboard[0] = (Key::Key0,true);
                        }else {
                            c.keyboard[0] =  (Key::Key0,false);
                        }
                        println!("w pressed");
                    },
                    _ => {
                        println!("{:?} pressed",key.code);
                    },
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
