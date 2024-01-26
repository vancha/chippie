use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Buffer, CrosstermBackend, Rect, Terminal},
    widgets::Widget,
};
use std::io::{stdout, Result};

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
    ///returns a value from ram
    fn get(self, index: u16) -> u16 {
        return ((self.bytes[index as usize] as u16) << 8)
            | self.bytes[(index + 1) as usize] as u16;
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
        }
    }
    fn set_index_register(&mut self, value: u16) {
        self.vindex = value;
    }
    fn get_index_register(&self) -> u16 {
        self.vindex
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
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,
    memory: RAM,
    registers: Registers,
    stack: Stack, //stack for keeping track of where to return to after subroutine, can go into 16 nested subroutines before stackoverflow
    stackpointer: u8, //only contains indexes to locations in the stack, so 0 through 15
}

//ratatui's 'widget' trait, to draw the display :)
impl Widget for CPU {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let idx = row as usize * DISPLAY_WIDTH + col as usize;
                buf.get_mut(col as u16, row as u16)
                    .set_char(match self.display[idx] {
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
                _ => panic!("what's going on {:#06x}", opcode),
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
            0xD => Instruction::DISPLAY {
                x: self.second_nibble(opcode),
                y: self.third_nibble(opcode),
                n: self.fourth_nibble(opcode),
            },
            0xF => match self.last_byte(opcode) {
                0x15 => Instruction::SetDelayTimerToX {
                    x: self.second_nibble(opcode),
                },
                0x1E => Instruction::AddXtoI {
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
                    print!("0xf, also unimplemented: {:0x}", opcode);
                    return Instruction::NOOP {};
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
            Instruction::NOOP {} => {
                println!("hi");
                //println!("nooping");
            }
            //00E0
            Instruction::ClearScreen => {
                self.display.iter_mut().for_each(|x| *x = false);
            }
            //00EE
            Instruction::ReturnFromSubroutine => {
                self.program_counter = self.stack.values[self.stackpointer as usize];
                self.stackpointer -= 1;
            }
            //0NNN
            Instruction::JUMP { nnn } => {
                self.program_counter = nnn;
            }
            //2NNN
            Instruction::CallSubroutineAtNNN { nnn } => {
                self.stackpointer += 1;
                self.stack.values[self.stackpointer as usize] = self.program_counter;
                self.program_counter = nnn;
            }
            //3XKK
            Instruction::SkipNextInstructionIfXIsKK { x, kk } => {
                if self.registers.get_register(x) == kk {
                    self.program_counter += 2;
                }
            }
            //4XKK
            Instruction::SkipNextInstructionIfXIsNotKK { x, kk } => {
                if self.registers.get_register(x) != kk {
                    self.program_counter += 2;
                }
            }
            //5XY0
            Instruction::SkipNextInstructionIfXIsY { x, y } => {
                if self.registers.get_register(x) == self.registers.get_register(y) {
                    self.program_counter += 2;
                }
            }
            //6XKK
            Instruction::LoadRegisterX { x, kk } => {
                self.registers.set_register(x, kk);
            }
            //7XKK
            Instruction::AddToRegisterX { x, kk } => {
                let tmp = self.registers.get_register(x) as u16 + kk as u16;
                //this can happen, if it does, the result is made to fit in a u8 again
                match tmp >= 255 {
                    _ => {
                        self.registers.set_register(x, tmp as u8);
                    }
                }
            }
            //8xy0
            Instruction::LoadRegisterXIntoY { x, y } => {
                self.registers
                    .set_register(x, self.registers.get_register(y));
            }
            //8xy1
            Instruction::LoadXOrYinX { x, y } => {
                self.registers.set_register(
                    x,
                    self.registers.get_register(x) | self.registers.get_register(y),
                );
            }
            //8xy2
            Instruction::LoadXAndYInX { x, y } => {
                self.registers.set_register(
                    x,
                    self.registers.get_register(x) & self.registers.get_register(y),
                );
            }
            //8xy3
            Instruction::LoadXXorYInX { x, y } => {
                self.registers.set_register(
                    x,
                    self.registers.get_register(x) ^ self.registers.get_register(y),
                );
            }
            //8xy4
            Instruction::AddYToX { x, y } => {
                let vx = self.registers.get_register(x);
                let vy = self.registers.get_register(y);
                let res = vy.overflowing_add(vx);
                self.registers.set_register(x, res.0);

                match res.1 {
                    true => {
                        self.registers.set_register(0xF, 1);
                    }
                    false => {
                        self.registers.set_register(0xF, 0);
                    }
                }
            }
            //8xy5
            Instruction::SubYFromX { x, y } => {
                let res = self
                    .registers
                    .get_register(x)
                    .overflowing_sub(self.registers.get_register(y));
                self.registers.set_register(x, res.0);
                match res.1 {
                    true => {
                        self.registers.set_register(0xF, 0);
                    }
                    false => {
                        self.registers.set_register(0xF, 1);
                    }
                }
            }

            //8xy6
            Instruction::ShiftXRight1 { x } => {
                let res = self.registers.get_register(x).overflowing_shr(1);
                self.registers.set_register(x, res.0);
                match res.1 {
                    true => {
                        self.registers.set_register(0xF, 1);
                    }
                    false => {
                        self.registers.set_register(0xF, 0);
                    }
                }
            }

            //8xyE
            Instruction::ShiftXLeft1 { x } => {
                let res = self.registers.get_register(x).overflowing_shl(1);
                self.registers.set_register(x, res.0);
                match res.1 {
                    true => {
                        self.registers.set_register(0xF, 1);
                    }
                    false => {
                        self.registers.set_register(0xF, 0);
                    }
                }
            }

            //8xy7
            Instruction::SubXFromY { x, y } => {
                let res = self
                    .registers
                    .get_register(y)
                    .overflowing_sub(self.registers.get_register(x));
                self.registers.set_register(x, res.0);
                match res.1 {
                    true => {
                        self.registers.set_register(0xF, 0);
                    }
                    false => {
                        self.registers.set_register(0xF, 1);
                    }
                }
            }
            //9XY0
            Instruction::SkipNextInstructionIfXIsNotY { x, y } => {
                if self.registers.get_register(x) != self.registers.get_register(y) {
                    self.program_counter += 2;
                }
            }
            //ANNN
            Instruction::SetIndexRegister { nnn } => {
                self.registers.set_index_register(nnn);
            }
            //DXYN
            Instruction::DISPLAY { x, y, n } => {
                let x_coordinate = self.registers.get_register(x) % DISPLAY_WIDTH as u8;
                let y_coordinate = self.registers.get_register(y) % DISPLAY_HEIGHT as u8;
                let sprite_start = self.registers.get_index_register() as usize;

                //clear 0xf register
                self.registers.set_register(0xF, 0);

                //height of sprite is 0 through n
                //this may fail when drawing out of bounds? maybe add a check for that
                for sprite_row in 0..n {
                    let sprite = self.memory.bytes[sprite_start + sprite_row as usize];
                    //width is always 8
                    for sprite_column in 0..8 {
                        let x = x_coordinate + sprite_column;
                        let y = y_coordinate + sprite_row;
                        let display_index = x as usize + DISPLAY_WIDTH * y as usize;
                        let value = sprite >> (7 - sprite_column) & 1 == 1;
                        self.display[display_index] = value;
                    }
                }
            }
            //fx15
            Instruction::SetDelayTimerToX { x } => {
                self.registers.delay_timer = self.registers.get_register(x);
            }
            //fx1E
            Instruction::AddXtoI { x } => {
                let added =
                    self.registers.get_index_register() + self.registers.get_register(x) as u16;
                self.registers.set_index_register(added);
            }
            Instruction::LoadBCDOfX { x } => {
                let store_index = self.registers.get_index_register() as usize;
                let value_to_convert = self.registers.get_register(x);
                self.memory.bytes[store_index] = value_to_convert / 100;
                self.memory.bytes[store_index + 1] = (value_to_convert % 100) / 10;
                self.memory.bytes[store_index + 2] = (value_to_convert % 100) % 10;
            }
            //fx55
            Instruction::Write0ThroughX { x } => {
                let start_storing_at = self.registers.get_index_register();

                for register in 0..x + 1 {
                    let register_value = self.registers.get_register(register);
                    self.memory.bytes[start_storing_at as usize + register as usize] =
                        register_value;
                }
            }
            //fx65
            Instruction::Load0ThroughX { x } => {
                let idx = self.registers.get_index_register();
                for i in 0..x + 1 {
                    self.registers
                        .set_register(i, self.memory.bytes[idx as usize + i as usize]);
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
    }

    fn new(rom: RomBuffer) -> Self {
        let mut memory = RAM::new();
        for (x, y) in rom.buffer.iter().enumerate() {
            memory.bytes[0x200 + x] = *y;
            //add all these bytes into memory, starting at 200
        }
        Self {
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            program_counter: 0x200,
            registers: Registers::new(),
            memory: memory,
            stack: Stack::new(),
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
    NOOP {},           //temporary instruction, used for development purposes
    JUMP { nnn: u16 }, //1nnn where nnn is a 12 bit value (lowest 12 bits of the instruction)
    ClearScreen,       //clears the screen, does not take any arguments
    AddToRegisterX { x: u8, kk: u8 },
    AddXtoI { x: u8 }, //set i = i + vx
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
    ReturnFromSubroutine, //pops the previous program_counter from the stack and makes it active
    SetIndexRegister { nnn: u16 }, //ANNN set index register I to nnn
    SkipNextInstructionIfXIsKK { x: u8, kk: u8 }, //skips the next instruction only if the register X holds the value kk
    SkipNextInstructionIfXIsNotKK { x: u8, kk: u8 }, //same as previous, except skips if register x does not hold value kk
    SkipNextInstructionIfXIsY { x: u8, y: u8 },
    SkipNextInstructionIfXIsNotY { x: u8, y: u8 },
    DISPLAY { x: u8, y: u8, n: u8 }, //DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
    SetDelayTimerToX { x: u8 },      //Fx15
    LoadBCDOfX { x: u8 },            //fx33
    Write0ThroughX { x: u8 },        //fx55
    Load0ThroughX { x: u8 },         //fx65
}

fn main() -> Result<()> {
    //creating a chip8 cpu object with a rom loaded
    let b = RomBuffer::new("./flagstest.ch8");
    let mut c = CPU::new(b);

    //folowing code is all for setting up the tui library "ratatui"
    stdout().execute(EnterAlternateScreen)?;
    //disable input and output processing by terminal itself (and buffering i guess?)
    enable_raw_mode()?;
    //lets the TUI library interface with a terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    //ratatui main loop, runs 60 times per second
    loop {
        //the "cyles per frame" metric for out chip8 cpu
        for _ in 0..=CYCLES_PER_FRAME {
            c.cycle();
        }
        //draw the ui
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(c, area);
        })?;
        //handle input events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Char('Q')
                {
                    break;
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
