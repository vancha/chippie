use std::cell::RefCell;
use std::rc::Rc;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::Framebuffer;
use crate::constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, NUM_KEYS, RAM_SIZE, ROM_START_ADDRESS};
use crate::instruction::Instruction;
use crate::ram::Ram;
use crate::registers::Registers;
use crate::rombuffer::RomBuffer;
use crate::stack::Stack;

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

/// The main cpu,
pub struct Cpu {
    /// A 2d array of booleans, representing the black and white pixels for the chip8 framebuffer
    framebuffer: Rc<RefCell<Framebuffer>>,
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,
    /// A list of "buttons", for the keyboard. set to true when pressed, false otherwise
    keyboard: [bool; NUM_KEYS as usize],
    /// The memory, stores the rom data when loaded from disk
    memory: Ram,
    /// A random number generator. Added for testability reaons as it allows to test all random instructions with a fixed seed
    rng: ChaCha8Rng,
    /// Used to check which quirks should be enabled or disabled
    quirks: Quirks,
    /// Registers 0x0 through 0xF
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

    ///Execute the instruction, for details on the instruction, check the instruction enum
    ///definition
    fn execute(&mut self, instruction: &Instruction) {
        match *instruction {
            Instruction::Noop => {
                //do nothing...
            }
            //00E0
            Instruction::ClearScreen => {
                self.framebuffer
                    .borrow_mut()
                    .iter_mut()
                    .for_each(|x| *x = [false; DISPLAY_WIDTH as usize]);
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
                //u8::from(fv)
                self.registers.set_register(0xf, u8::from(fv));
            }

            //8xy5
            Instruction::SubYFromX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);

                let (res, fv) = vx.overflowing_sub(vy);
                self.registers.set_register(x, res);
                self.registers.set_register(0xf, u8::from(!fv));
            }

            //8xy6
            Instruction::ShiftXRight1 { x } => {
                let vx = self.registers.get_register(x);
                let vf = u8::from(vx & 1 == 1);

                self.registers.set_register(x, vx.overflowing_shr(1).0);
                self.registers.set_register(0xF, vf);
            }

            //8xyE
            Instruction::ShiftXLeft1 { x } => {
                let vx = self.registers.get_register(x);
                let fv = (u16::from(vx) >> 7) & 1;
                let res = self.registers.get_register(x).wrapping_shl(1);

                self.registers.set_register(x, res);
                self.registers.set_register(0xf, u8::try_from(fv).unwrap());
            }
            //8xy7
            Instruction::SubXFromY { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                let (res, fv) = vy.overflowing_sub(vx);
                self.registers.set_register(x, res);
                self.registers.set_register(0xf, u8::from(!fv));
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
                let v0 = u16::from(self.registers.get_register(0) & 0xf); //(self.registers.get_register(0) & 0xf) as u16;
                self.program_counter = nnn + v0;
            }
            //cxkk
            Instruction::SetXToRandom { x, kk } => {
                let random_byte: u8 = self.rng.random();
                self.registers.set_register(x, random_byte & kk);
            }
            //DXYN
            Instruction::Display { x, y, n } => {
                //drawing at (start_x, start_y) on the framebuffer, wraps around if out of bounds
                let start_x = (self.registers.get_register(x) % DISPLAY_WIDTH) as usize;
                let start_y = (self.registers.get_register(y) % DISPLAY_HEIGHT) as usize;

                let sprite_start = self.registers.get_index_register() as usize;
                self.registers.set_register(0xF, 0);

                //move over all rows of the sprite (it has n rows)
                for sprite_row in 0..n as usize {
                    if sprite_start + sprite_row >= RAM_SIZE as usize {
                        return;
                    }
                    //bytes[sprite_start + sprite_row];
                    let sprite = self.memory.bytes[sprite_start + sprite_row];
                    //what is the sprite?
                    for sprite_column in 0..8 {
                        let pixel_row = start_x + sprite_column;
                        let pixel_column = start_y + sprite_row;

                        let sprite_pixel_set = sprite >> (7 - sprite_column) & 1 == 1;

                        //check so as to *not* draw out of bounds of the framebuffer
                        if pixel_row < u16::from(DISPLAY_WIDTH).into()
                            && u16::try_from(pixel_column).unwrap() < u16::from(DISPLAY_HEIGHT)
                        {
                            let mut framebuffer = self.framebuffer.borrow_mut();
                            if framebuffer[pixel_column][pixel_row] && sprite_pixel_set {
                                self.registers.set_register(0xf, 1);
                            }
                            framebuffer[pixel_column][pixel_row] ^= sprite_pixel_set;
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
                let vx = u16::from(self.registers.get_register(x));
                let vi = self.registers.get_index_register();
                let added = vi + vx;

                self.registers.set_index_register(added);
            }
            //fx29
            Instruction::SetIToSpriteX { x } => {
                let vx = u16::from(self.registers.get_register(x) * 5);
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

                for register in 0..=x {
                    let register_value = self.registers.get_register(register);
                    self.memory.set(vi + u16::from(register), register_value);
                }
            }
            //fx65
            Instruction::Load0ThroughX { x } => {
                let vi = self.registers.get_index_register();
                for i in 0..=x {
                    self.registers
                        .set_register(i, self.memory.get_byte(vi + u16::from(i)));
                }
            }
        }
    }
    pub fn get_pressed_key(&self) -> Option<usize> {
        self.keyboard
            .iter()
            .position(|button_pressed| *button_pressed)
    }

    /// Set key's state
    pub fn set_key_state(&mut self, key: u8, state: bool) {
        assert!(key <= NUM_KEYS);
        self.keyboard[key as usize] = state;
    }

    /// A single cpu cycle, fetches, decodes, executes opcodes and
    /// decrements the timers if relevant. also updates the program counter
    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        self.program_counter += 2;

        let instruction = Instruction::new(opcode);
        self.execute(&instruction);

        self.registers.decrement_sound_timer();
        self.registers.decrement_delay_timer();
    }

    /// Creates a new cpu object, with the contents of a rom file loaded in to memory
    pub fn new(rom: &RomBuffer, framebuffer: Rc<RefCell<Framebuffer>>) -> Self {
        let program_counter = ROM_START_ADDRESS;
        let registers = Registers::default();
        let keyboard = [false; 16];
        let quirks = Quirks::default();
        let rng = ChaCha8Rng::seed_from_u64(2);
        let mut memory = Ram::with_fonts();

        for (x, y) in rom.contents().iter().enumerate() {
            memory.set(ROM_START_ADDRESS + x as u16, *y);
        }

        let stack = Stack::default();

        Self {
            framebuffer,
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

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::NUM_REGISTERS;

    fn create_framebuffer() -> Rc<RefCell<Framebuffer>> {
        Rc::new(RefCell::new(
            [[false; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
        ))
    }

    #[test]
    fn it_can_initialize() {
        let buffer = RomBuffer::new("assets/1-chip8-logo.8o");
        let cpu = Cpu::new(&buffer, create_framebuffer());
        assert!(cpu.program_counter == ROM_START_ADDRESS);
    }

    #[test]
    fn it_can_fetch_instruction() {
        let buffer = RomBuffer::new("assets/1-chip8-logo.8o");
        let cpu = Cpu::new(&buffer, create_framebuffer());
        assert!(cpu.fetch() == 0x2320);
    }

    // instructions in order of https://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf
    #[test]
    fn executes_00E0() {
        // Clears the framebuffer
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x00, 0xE0]),
            create_framebuffer(),
        );
        cpu.framebuffer.borrow_mut()[0][0] = true;
        cpu.cycle();
        assert!(!cpu.framebuffer.borrow()[0][0]);
    }

    #[test]
    fn executes_00EE() {
        // Return from a subroutine
        // sets the counter to the address at the top of the stack, and subtracts 1 from the stack pointer
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x00, 0xEE]),
            create_framebuffer(),
        );
        cpu.stack.set(0, 0x201);
        cpu.stackpointer = 1;
        cpu.cycle();
        assert!(cpu.stackpointer == 0);
        assert!(cpu.program_counter == 0x201);
    }

    #[test]
    fn executes_1NNN() {
        // Jumps to location nnn, this should set the program counter to nnn
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x11, 0x23]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert_eq!(cpu.program_counter, 0x123);
    }

    #[test]
    fn executes_2NNN() {
        // Call subroutine at nnn, this should:
        // 1. increment the stack pointer
        // 2. put the current program counter at the top of the stack
        // 3. sets the program counter to nnn
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x21, 0x23]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.stackpointer == 1);
        //In a cycle the program counter gets updated before it is pushed to the stack
        assert!(cpu.stack.get(0) == 0x202);
        assert!(cpu.program_counter == 0x123);
    }

    #[test]
    fn executes_3XKK() {
        //Should increment the program counter by two if  register VX is equal to NN
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x31, 0x00]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.program_counter == 0x204);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x31, 0x01]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.program_counter == 0x202);
    }

    #[test]
    fn executes_4Xkk() {
        //Should increment the program counter by two if  register VX is not equal to NN
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x41, 0x00]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.program_counter == 0x202);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x41, 0x01]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.program_counter == 0x204);
    }

    #[test]
    fn executes_5XY0() {
        //Should increment the program counter by two if register vs equals register vy
        //here regsiter x and y are both 0, which should update the pc to 0x204
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x51, 0x20]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.program_counter == 0x204);

        //Here register 1 will be set to 5, so it should leave the pc at 0x202
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x51, 0x20]),
            create_framebuffer(),
        );
        cpu.registers.set_register(1, 5);
        cpu.cycle();
        assert!(cpu.program_counter == 0x202);
    }

    #[test]
    fn executes_6XKK() {
        //Should put the value KK in to register X
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x60, 0x22]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.registers.get_register(0) == 0x22);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x61, 0x23]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 0x23);
    }

    #[test]
    fn executes_7XKK() {
        //Should put the value KK plus the current value of register x in to register X
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x71, 0x05]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 0x4);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 0x09);
    }

    #[test]
    fn executes_8XY0() {
        //Should store the value of register y in to register x
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x20]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 0x4);
        cpu.cycle();
        assert!(cpu.registers.get_register(2) == 0x4);
    }

    #[test]
    fn executes_8XY1() {
        //Should store the value of register y ORED with whatever is in register y in to register x
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x21]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 4);
        cpu.registers.set_register(0x1, 2);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == (4 | 2));
    }

    #[test]
    fn executes_8XY2() {
        //Should store the value of register y ANDed with whatever is in register y in to register x
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x22]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 4);
        cpu.registers.set_register(0x1, 2);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == (4 & 2));
    }

    #[test]
    fn executes_8XY3() {
        //Should store the value of register y XORed with whatever is in register y in to register x
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x23]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 4);
        cpu.registers.set_register(0x1, 2);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == (4 ^ 2));
    }

    #[test]
    fn executes_8XY4() {
        //Should store the value of register y ADDED to whatever is in register y in to register x
        //if the value is bigger than 8 bits (i.e 255), register f should be set to 1, 0 otherwise, and only the lowest
        //8 bit of the result should be kept and stored in register x
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x24]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 200);
        cpu.registers.set_register(0x1, 1);
        cpu.cycle();

        assert!(cpu.registers.get_register(1) == (200 + 1));
        assert!(cpu.registers.get_register(0xf) == 0);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x24]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x2, 200);
        cpu.registers.set_register(0x1, 60);
        cpu.cycle();
        //This shows an overflow (200 + 60 is bigger than 8 bits), only the last 8 bits should be saved (so AND'ed with 0xff)
        assert!(cpu.registers.get_register(1) as u16 == (200 + 60) & 0xff);
        //and the carry flag should be set
        assert!(cpu.registers.get_register(0xf) == 1);
    }

    #[test]
    fn executes_8XY5() {
        //Should store the value of register y subtracted from whatever is in register y in to register x
        //if an underflow occurs, register f is set to 0, otherwise its 1. So the opposite of what you'd expect
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x25]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 10);
        cpu.registers.set_register(0x2, 5);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 10 - 5);
        assert!(cpu.registers.get_register(0xf) == 1);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x25]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 5);
        cpu.registers.set_register(0x2, 10);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 5u8.overflowing_sub(10).0);
        assert!(cpu.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XY6() {
        //Should store the value of register x shifted right by one in register x
        //sets register f to 1 if the least significant bit of vx is 1, otherwise it sets it to 0
        //then vx is divided by 2?
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x26]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 17);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 8);
    }

    #[test]
    fn executes_8XY7() {
        // Should store the value of register x subtracted from the value in register y, inside register x. register f is said when we didn't borrow.
        // again, opposite of what you'd expect.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x27]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 2);
        cpu.registers.set_register(0x2, 10);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 8);
        assert!(cpu.registers.get_register(0xf) == 1);

        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x27]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x1, 10);
        cpu.registers.set_register(0x2, 2);
        cpu.cycle();
        assert!(cpu.registers.get_register(1) == 248);
        assert!(cpu.registers.get_register(0xf) == 0);
    }

    #[test]
    fn executes_8XYE() {
        // Set register x equal to itself shifted left by one. if msb of x is 1, then set VF. If not,
        // unset it. Afterwards, multiply the value at register x by 2. (not sure if i get that right, shl == multiply by 2)
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0x81, 0x2E]),
            create_framebuffer(),
        );
        //the number 0xff has the most significant bit set to 1, so vf must be set when done
        cpu.registers.set_register(0x1, 0xff);
        cpu.cycle();
        //Check if it correctly sets vf based on the most significant bit
        assert!(cpu.registers.get_register(0xF) == 0x1);
        //Check if it correctly calculates the result of the instruction
        assert_eq!(cpu.registers.get_register(0x1), 0xff << 1);
    }

    #[test]
    fn executes_ANNN() {
        // Directly sets the index register to NNN
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xA1, 0x23]),
            create_framebuffer(),
        );
        cpu.cycle();
        assert!(cpu.registers.get_index_register() == 0x123);
    }

    #[test]
    fn executes_BNNN() {
        // Directly sets the index register to NNN
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xB3, 0x00]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0, 0x5);
        cpu.cycle();
        assert!(cpu.program_counter == 0x5 + 0x300);
    }

    #[test] //This test is disabled because i have no idea how to test random numbers
    fn executes_CXKK() {
        // Set Vx = random byte AND kk. The interpreter generates a random number from 0 to 255, which is then
        // ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xC0, 0xff]),
            create_framebuffer(),
        );
        cpu.cycle();
        let random_number = cpu.registers.get_register(0);
        assert_eq!(random_number, 197);

        //here the ANDed number is 0, so the result is zero too
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xC0, 0x00]),
            create_framebuffer(),
        );
        cpu.cycle();
        let random_number = cpu.registers.get_register(0);
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
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![
                0xD1, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
            ]),
            create_framebuffer(),
        );
        //this should set both x and y to 2
        cpu.registers.set_register(1, 2);
        cpu.registers.set_register(2, 2);

        //sprite data starts at (keep in mind program data starts at 0x200, aad locations before that are pointing at font data probably)
        //this points to the ninth byte (0xff) in our rom as the start of our sprite data
        cpu.registers.set_index_register(ROM_START_ADDRESS + 9);

        //Given all this, chip8 should put 8 ones at (2,2) on the display
        cpu.cycle();
        let byte_of_ones = cpu.framebuffer.borrow_mut()[2];
        let mut what_it_should_look_like = [false; 64];
        what_it_should_look_like[..10]
            .copy_from_slice(&[false, false, true, true, true, true, true, true, true, true]); //this is what the second column should look like
        assert_eq!(byte_of_ones, what_it_should_look_like);
    }

    #[test]
    fn executes_EX9E() {
        // Skip next instruction if key with the value of Vx is pressed. Checks the keyboard, and if the key corresponding
        // to the value of Vx is currently in the down position, PC is increased by 2.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xE0, 0x9E]),
            create_framebuffer(),
        );
        // location will be 0 (since that's default value for register 0)
        cpu.keyboard[0] = true;
        cpu.cycle();
        //should be incremented by 4 rather than two if button is down
        assert!(cpu.program_counter == 0x200 + 4);
    }

    #[test]
    fn executes_ExA1() {
        // Skip next instruction if key with the value of Vx is not pressed. Checks the keyboard, and if the key
        // corresponding to the value of Vx is currently in the up position, PC is increased by 2.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xE0, 0xA1]),
            create_framebuffer(),
        );
        cpu.cycle();
        //should be incremented by 4 rather than two if button is not down
        assert!(cpu.program_counter == 0x200 + 4);
    }

    #[test]
    fn executes_Fx07() {
        //Set Vx = delay timer value. The value of DT is placed into Vx.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF0, 0x07]),
            create_framebuffer(),
        );
        cpu.registers.set_delay_timer(0x10);
        cpu.cycle();
        //should be equal to delay timer
        assert!(cpu.registers.get_register(0x0) == 0x10);
    }

    #[test]
    fn executes_Fx0A() {
        //Wait for a key press, store the value of the key in Vx. All execution stops until a key is pressed, then the
        //value of that key is stored in Vx.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF0, 0x0A]),
            create_framebuffer(),
        );
        cpu.cycle();
        cpu.cycle();
        cpu.cycle();
        //should be equal to delay timer
        assert!(cpu.program_counter == 0x200);
    }

    #[test]
    fn executes_Fx15() {
        //- LD DT, Vx
        //Set delay timer = Vx. Delay Timer is set equal to the value of Vx.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xf0, 0x15]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0, 125);
        cpu.cycle();
        let val = cpu.registers.get_delay_timer();
        //the value is one less than the actual value, because during the cycle the delay timer
        //also gets decremented by one..
        assert!(val == 124);
    }

    #[test]
    fn executes_Fx18() {
        //- LD ST, Vx
        //Set sound timer = Vx. Sound Timer is set equal to the value of Vx.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xf0, 0x18]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0, 125);
        cpu.cycle();
        let val = cpu.registers.get_sound_timer();
        //the value is one less than the actual value, because during the cycle the delay timer
        //also gets decremented by one..
        assert!(val == 124);
    }

    #[test]
    fn executes_Fx1E() {
        // - ADD I, Vx
        //Set I = I + Vx. The values of I and Vx are added, and the results are stored in I.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF6, 0x1E]),
            create_framebuffer(),
        );
        cpu.registers.set_index_register(0x6);
        cpu.registers.set_register(0x6, 6);
        cpu.cycle();
        assert!(cpu.registers.get_index_register() == 12);
    }

    #[test]
    fn executes_Fx29() {
        // - LD F, Vx
        //Set I = location of sprite for digit Vx. The value of I is set to the location for the hexadecimal sprite
        //corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal
        //font. To obtain this value, multiply VX by 5 (all font data stored in first 80 bytes of memory).
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF0, 0x29]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x0, 0x6);
        cpu.cycle();
        assert!(cpu.registers.get_index_register() == 6 * 5);
    }

    #[test]
    fn executes_Fx33() {
        // - LD B, Vx
        //Store BCD representation of Vx in memory locations I, I+1, and I+2. The interpreter takes the decimal
        //value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and
        //the ones digit at location I+2.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF0, 0x33]),
            create_framebuffer(),
        );
        cpu.registers.set_register(0x0, 123);
        cpu.registers.set_index_register(220);
        cpu.cycle();
        assert!(cpu.memory.get_byte(cpu.registers.get_index_register()) == 1);
        assert!(cpu.memory.get_byte(cpu.registers.get_index_register() + 1) == 2);
        assert!(cpu.memory.get_byte(cpu.registers.get_index_register() + 2) == 3);
    }

    #[test]
    fn executes_Fx55() {
        // - LD [I], Vx
        //Stores V0 to VX in memory starting at address I. I is not changed.
        let register_value = 0xFF;
        for x in 0..16 {
            for index_register_value in 0..RAM_SIZE - x {
                let mut cpu = Cpu::new(
                    &RomBuffer::from_bytes(vec![(x | 0xF0) as u8, 0x55]),
                    create_framebuffer(),
                );
                // Fill registers with the expected values
                for register in 0..NUM_REGISTERS {
                    cpu.registers.set_register(register, register_value);
                }
                cpu.registers.set_index_register(index_register_value);

                // Run the emulator to see the effect
                cpu.cycle();

                // Check if every byte inside RAM corresponds to the expected value
                for offset in 0..x {
                    assert!(cpu.memory.get_byte(index_register_value + offset) == register_value);
                }

                // Check if VI changed
                assert!(cpu.registers.get_index_register() == index_register_value)
            }
        }
    }

    #[test]
    fn executes_Fx65() {
        // - LD Vx, [I]
        //Fills V0 to VX with values from memory starting at address I. I is then set to I + x + 1.
        let mut cpu = Cpu::new(
            &RomBuffer::from_bytes(vec![0xF3, 0x65, 0x01, 0x02, 0x03]),
            create_framebuffer(),
        );
        //sets the index register to point to the 0x01 in the ROM
        cpu.registers.set_index_register(ROM_START_ADDRESS + 2);
        //copies the values 0x01, 0x02 and 0x03 from memory to the registers
        cpu.cycle();
        //checks if the index register is unaffected as it should be (without quirks support)
        assert!(cpu.registers.get_index_register() == ROM_START_ADDRESS + 2);
        //checks if the registers have been set correctly
        assert!(cpu.registers.get_register(0) == 0x01);
        assert!(cpu.registers.get_register(1) == 0x02);
        assert!(cpu.registers.get_register(2) == 0x03);
    }
}
