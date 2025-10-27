use crate::constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, RAM_SIZE};
use crate::instruction::*;
use crate::ram::*;
use crate::registers::*;
use crate::rombuffer::*;
use crate::stack::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

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
pub struct Cpu {
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,
    /// A list of "buttons", for the keyboard. set to true when pressed, false otherwise
    keyboard: [bool; 16],
    memory: Ram,
    rng: ChaCha8Rng,
    quirks: Quirks,
    registers: Registers,
    stack: Stack,
    /// Only contains indexes to locations in the stack, so 0 through 15
    stackpointer: u8,
}

impl Cpu {
    /// Returns two bytes from memory at the location where the program counter currently points to
    fn fetch(&self) -> u16 {
        self.memory.get_opcode(self.program_counter)
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
        println!("instruction: {:?}", instruction);
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
                self.program_counter = self.stack.get(self.stackpointer); //self.stack.values[self.stackpointer as usize];
            }
            //1NNN
            Instruction::Jump { nnn } => {
                self.program_counter = nnn;
            }
            //2NNN
            Instruction::CallSubroutineAtNNN { nnn } => {
                self.stack.set(self.stackpointer, self.program_counter); //.values[self.stackpointer as usize] = self.program_counter;
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

                let (tmp, _overflow) = vx.overflowing_add(kk);
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
                println!("VF is {}", fv);
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
                let random_byte: u8 = self.rng.random();
                self.registers.set_register(x, random_byte & kk);
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
                    let sprite = self.memory.bytes[sprite_start + sprite_row]; //bytes[sprite_start + sprite_row];
                                                                               //what is the sprite?
                    println!("The sprite is {:#b}", sprite);
                    for sprite_column in 0..8 {
                        let pixel_row = start_x + sprite_column;
                        let pixel_column = start_y + sprite_row;

                        let sprite_pixel_set = sprite >> (7 - sprite_column) & 1 == 1;

                        //check so as to *not* draw out of bounds of the display
                        if pixel_row < (DISPLAY_WIDTH as u16).into()
                            && (pixel_column as u16) < (DISPLAY_HEIGHT as u16)
                        {
                            if self.display[pixel_column as usize][pixel_row as usize]
                                && sprite_pixel_set
                            {
                                self.registers.set_register(0xf, 1);
                            }
                            self.display[pixel_column as usize][pixel_row as usize] ^=
                                sprite_pixel_set;
                        }
                    }
                }
            }
            //exa1
            Instruction::SkipIfVxNotPressed { x } => {
                //@TODO: check behavior
                if !self.keyboard[x as usize] {
                    self.program_counter += 2;
                }
            }
            //ex9e
            Instruction::SkipIfVxPressed { x } => {
                //@TODO: check behavior
                if self.keyboard[x as usize] {
                    self.program_counter += 2;
                }
            }
            //fx0a
            Instruction::WaitForKeyPressed { x } => {
                match self.get_pressed_key() {
                    //@TODO: check behavior
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
                let vx = (self.registers.get_register(x) * 5) as u16;
                //the sprite at *index* x, not location x.
                self.registers.set_index_register(vx);
            }
            Instruction::LoadBCDOfX { x } => {
                let vx = self.registers.get_register(x);
                let store_index = self.registers.get_index_register();
                self.memory.set(store_index, vx / 100);
                self.memory.set(store_index + 1, (vx % 100) / 10);
                self.memory.set(store_index + 2, (vx % 100) % 10);
            }
            //fx55
            Instruction::Write0ThroughX { x } => {
                let vi = self.registers.get_index_register();

                for register in 0..x + 1 {
                    let register_value = self.registers.get_register(register);
                    self.memory.set(vi + register as u16, register_value);
                }
            }
            //fx65
            Instruction::Load0ThroughX { x } => {
                let vi = self.registers.get_index_register();
                for i in 0..x + 1 {
                    self.registers
                        .set_register(i, self.memory.get_byte(vi + i as u16));
                }
            }
        }
    }
    /// A nibble is 4 bits, so this returns the first 4 bits of an opcode
    fn first_nibble(&self, opcode: u16) -> u8 {
        ((opcode >> 12) & 0xf) as u8
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

    //this should be part of an interface somehow, maybe a trait that lets external programs set the keys for the emulator
    fn set_pressed_key(&mut self, key: u8) {
        if !self.keyboard[key] {
            self.keyboard[key] = true;
        }
    }

    pub fn get_display_contents(&self) -> [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        self.display
    }
    /// A single cpu cycle, fetches, decodes, executes opcodes and
    /// decrements the timers if relevant. also updates the program_counter
    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        self.program_counter += 2;

        let instruction = self.decode(opcode);
        self.execute(instruction);

        self.registers.decrement_sound_timer();
        self.registers.decrement_delay_timer();
    }

    /// Creates a new cpu object, with the contents of a rom file loaded in to memory
    pub fn new(rom: RomBuffer) -> Self {
        let display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        let program_counter = 0x200;
        let registers = Registers::new();
        let keyboard = [false; 16];
        let quirks = Quirks::default();
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut memory = Ram::with_fonts();
        for (x, y) in rom.contents().iter().enumerate() {
            memory.set(0x200 + x as u16, *y);
        }

        let stack = Stack::new();

        Self {
            display,
            program_counter,
            registers,
            keyboard,
            quirks,
            rng,
            memory,
            stack,
            stackpointer: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

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
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x00, 0xE0]));
        instance.display[0][0] = true;
        instance.cycle();
        assert!(instance.display[0][0] == false);
    }
    #[test]
    fn executes_00EE() {
        // Return from a subroutine
        // sets the counter to the address at the top of the stack, and subtracts 1 from the stack pointer
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x00, 0xEE]));
        instance.stack.set(0, 0x201);
        instance.stackpointer = 1;
        instance.cycle();
        assert!(instance.stackpointer == 0);
        assert!(instance.program_counter == 0x201);
    }

    #[test]
    fn executes_1NNN() {
        // Jumps to location nnn, this should set the program counter to nnn
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x11, 0x23]));
        instance.cycle();
        assert_eq!(instance.program_counter == 0x123, true);
    }

    #[test]
    fn executes_2NNN() {
        // Call subroutine at nnn, this should:
        // 1. increment the stack pointer
        // 2. put the current program counter at the top of the stack
        // 3. sets the program counter to nnn
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x21, 0x23]));
        instance.cycle();
        assert!(instance.stackpointer == 1);
        //I'm not sure why this isn't 0x200
        assert!(instance.stack.get(0) == 0x202);
        assert!(instance.program_counter == 0x123);
    }

    #[test]
    fn executes_3XKK() {
        //Should increment the program counter by two if  register VX is equal to NN
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x31, 0x00]));
        instance.cycle();
        assert!(instance.program_counter == 0x204);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x31, 0x01]));
        instance.cycle();
        assert!(instance.program_counter == 0x202);
    }

    #[test]
    fn executes_4Xkk() {
        //Should increment the program counter by two if  register VX is not equal to NN
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x41, 0x00]));
        instance.cycle();
        assert!(instance.program_counter == 0x202);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x41, 0x01]));
        instance.cycle();
        assert!(instance.program_counter == 0x204);
    }

    #[test]
    fn executes_5XY0() {
        //Should increment the program counter by two if register vs equals register vy
        //here regsiter x and y are both 0, which should update the pc to 0x204
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x51, 0x20]));
        instance.cycle();
        assert!(instance.program_counter == 0x204);

        //Here register 1 will be set to 5, so it should leave the pc at 0x202
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x51, 0x20]));
        instance.registers.set_register(1, 5);
        instance.cycle();
        assert!(instance.program_counter == 0x202);
    }

    #[test]
    fn executes_6XKK() {
        //Should put the value KK in to register X
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x60, 0x22]));
        instance.cycle();
        assert!(instance.registers.get_register(0) == 0x22);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x61, 0x23]));
        instance.cycle();
        assert!(instance.registers.get_register(1) == 0x23);
    }

    #[test]
    fn executes_7XKK() {
        //Should put the value KK plus the current value of register x in to register X
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x71, 0x05]));
        instance.registers.set_register(0x1, 0x4);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 0x09);
    }

    #[test]
    fn executes_8XY0() {
        //Should store the value of register y in to register x
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x20]));
        instance.registers.set_register(0x2, 0x4);
        instance.cycle();
        assert!(instance.registers.get_register(2) == 0x4);
    }

    #[test]
    fn executes_8XY1() {
        //Should store the value of register y ORED with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x21]));
        instance.registers.set_register(0x2, 4);
        instance.registers.set_register(0x1, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == (4 | 2));
    }

    #[test]
    fn executes_8XY2() {
        //Should store the value of register y ANDed with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x22]));
        instance.registers.set_register(0x2, 4);
        instance.registers.set_register(0x1, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == (4 & 2));
    }

    #[test]
    fn executes_8XY3() {
        //Should store the value of register y XORed with whatever is in register y in to register x
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x23]));
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
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x24]));
        instance.registers.set_register(0x2, 200);
        instance.registers.set_register(0x1, 1);
        instance.cycle();

        assert!(instance.registers.get_register(1) == (200 + 1));
        assert!(instance.registers.get_register(0xf) == 0);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x24]));
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
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x25]));
        instance.registers.set_register(0x1, 10);
        instance.registers.set_register(0x2, 5);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 10 - 5);
        assert!(instance.registers.get_register(0xf) == 1);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x25]));
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
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x26]));
        instance.registers.set_register(0x1, 17);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 8);
        //assert!(instance.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XY7() {
        // Should store the value of register x subtracted from the value in register y, inside register x. register f is said when we didn't borrow.
        // again, opposite of what you'd expect.
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x27]));
        instance.registers.set_register(0x1, 2);
        instance.registers.set_register(0x2, 10);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 8);
        assert!(instance.registers.get_register(0xf) == 1);

        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x27]));
        instance.registers.set_register(0x1, 10);
        instance.registers.set_register(0x2, 2);
        instance.cycle();
        assert!(instance.registers.get_register(1) == 248);
        assert!(instance.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XYE() {
        // Set register x equal to itself shifted left by one. if msb of x is 1, then set VF. If not,
        // unset it. Afterwards, multiply the value at register x by 2. (not sure if i get that right, shl == multiply by 2)
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0x81, 0x2E]));
        //the number 0xff has the most significant bit set to 1, so vf must be set when done
        instance.registers.set_register(0x1, 0xff);
        instance.cycle();
        //Check if it correctly sets vf based on the most significant bit
        assert!(instance.registers.get_register(0xF) == 0x1);
        //Check if it correctly calculates the result of the instruction
        assert_eq!(instance.registers.get_register(0x1), 0xff << 1);
    }

    #[test]
    fn executes_ANNN() {
        // Directly sets the index register to NNN
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0xA1, 0x23]));
        instance.cycle();
        assert!(instance.registers.get_index_register() == 0x123);
    }

    #[test]
    fn executes_BNNN() {
        // Directly sets the index register to NNN
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0xB3, 0x00]));
        instance.registers.set_register(0, 0x5);
        instance.cycle();
        assert!(instance.program_counter == 0x5 + 0x300);
    }

    #[test] //This test is disabled because i have no idea how to test random numbers
    fn executes_CXKK() {
        // Set Vx = random byte AND kk. The interpreter generates a random number from 0 to 255, which is then
        // ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0xC0, 0xff]));
        instance.cycle();
        let random_number =  instance.registers.get_register(0);
        assert_eq!(random_number, 197);

        //here the ANDed number is 0, so the result is zero too
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0xC0, 0x00]));
        instance.cycle();
        let random_number =  instance.registers.get_register(0);
        assert_eq!(random_number, 0);
    }

    #[test]
    fn executes_DXYN() {
        //Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
        //The interpreter reads n bytes from memory, starting at the address stored in I.
        //These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
        //Sprites are XORed onto the existing screen.
        //If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
        //If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
        //See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.

        //0xD123 should make a 1-byte tall sprite sprite (n == 1), (x == 1 and y == 2)
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![
            0xD1, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
        ]));
        //this should set both x and y to 2
        instance.registers.set_register(1, 2);
        instance.registers.set_register(2, 2);

        //sprite data starts at (keep in mind program data starts at 0x200, aad locations before that are pointing at font data probably)
        //this points to the ninth byte (0xff) in our rom as the start of our sprite data
        instance.registers.set_index_register(0x200 + 9);

        //Given all this, chip8 should put 8 ones at (2,2) on the display
        instance.cycle();
        let byte_of_ones = instance.get_display_contents()[2];
        let mut what_it_should_look_like = [false; 64];
        what_it_should_look_like[..10]
            .copy_from_slice(&[false, false, true, true, true, true, true, true, true, true]); //this is what the second column should look like
        assert_eq!(byte_of_ones, what_it_should_look_like);
    }

    #[test]
    fn executes_EX9E() {
        // Skip next instruction if key with the value of Vx is pressed. Checks the keyboard, and if the key corresponding
        // to the value of Vx is currently in the down position, PC is increased by 2.
        let mut instance = Cpu::new(RomBuffer::from_bytes(vec![0xE0, 0x9E]));
        instance.cycle();
        println!("random_byte: {:?}", instance.registers.get_register(0));
        assert!(false);
    }
}

/*

*/
