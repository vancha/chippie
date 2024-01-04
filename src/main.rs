const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;//in bytes :)

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

struct Stack {
    values: [u16; 16],
}
impl Stack {
    fn new() -> Self {
        Stack { values: [0; 16] }
    }
}

///# The chip8 cpu, contains the ram, registers, stack and display
///
/// has a run method that's intended to be called 500 times a second,
///also has a display method, which is intended to be called 60 times a second
struct CPU {
    ///The monochrome display
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    ///Program counter, used to keep track of what to fetch,decode and execute from ram, initialized at 0x200
    program_counter: u16,

    memory: RAM,
    registers: Registers,
    stack: Stack, //stack for keeping track of where to return to after subroutine, can go into 16 nested subroutines before stackoverflow
    stackpointer: u8, //only contains indexes to locations in the stack, so 0 through 15
}

impl CPU {
    fn fetch(&self, ram: &RAM) -> u16 {
        ram.get(self.program_counter)
    }

    fn decode(&self, opcode: u16) -> Instruction {
        match self.xooo(opcode) {
            0x0 => match self.ooxx(opcode) {
                0xE0 => Instruction::ClearScreen,
                _ => panic!("what's going on {:#06x}", opcode),
            },
            0x1 => Instruction::JUMP { nnn: self.oxxx(opcode) },
            0x6 => Instruction::LoadRegisterVx { x: self.oxoo(opcode), kk: self.ooxx(opcode) },
            0x7 => Instruction::AddToRegister { x: self.oxoo(opcode),kk: self.ooxx(opcode) },
            0xA => Instruction::SetIndexRegister{ nnn: self.oxxx(opcode) },
            0xD => Instruction::DISPLAY { x: self.oxoo(opcode), y: self.ooxo(opcode), n: self.ooox(opcode) },
            _ => {
                panic!("cannot decode,opcode not implemented. ")
            }
        }
    }
    ///Execute the instruction, for details on the instruction, check the instruction enum
    ///definition
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            ///1nnn
            Instruction::JUMP  { nnn: x } => {
                self.program_counter = x;
            }
            ///7xkk
            Instruction::AddToRegister { x: x, kk: y} => {
                let tmp = self.registers.get_register(x) + y;
                self.registers.set_register(x, tmp);
            }
            Instruction::ClearScreen => {
                self.display.iter_mut().for_each(|x| *x = false);
            }
            Instruction::LoadRegisterVx { x: x, kk: y } => {
                self.registers.set_register(x, y);
            }
            ///
            Instruction::SetIndexRegister { nnn: x } => {
                self.registers.set_index_register(x);
            }
            ///DXYN
            Instruction::DISPLAY { x: vx, y: vy, n: n } => {
                let x_coordinate = self.registers.get_register(vx) % DISPLAY_WIDTH as u8;
                let y_coordinate = self.registers.get_register(vy) % DISPLAY_HEIGHT as u8;
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
        }
    }

    fn xooo(&self, code: u16) -> u8 {
        ((code >> 12) & 0xF) as u8
    }
    fn oxoo(&self, code: u16) -> u8 {
        ((code >> 8) & 0xf) as u8
    }
    fn ooxo(&self, code: u16) -> u8 {
        ((code >> 4) & 0xf) as u8
    }
    fn ooox(&self, code: u16) -> u8 {
        (code as u8) & 0xf
    }
    fn ooxx(&self, code: u16) -> u8 {
        (code & 0xff) as u8
    }
    fn oxxx(&self, code: u16) -> u16 {
        code & 0xfff
    }

    ///Shows contents of the display, ██ for set pixels, and two spaces for unset ones 
    fn display(&self) {
        //the special "clear screen" character for linux terminals
        print!("{}[2J", 27 as char);
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let idx = row as usize * DISPLAY_WIDTH + col as usize;
                print!("{}", if self.display[idx] { "██" } else { "  " })
            }
            println!("");
        }
    }

    fn cycle(&mut self) {
        //do this part 500 times a second
        let opcode = self.fetch(&self.memory);
        self.program_counter = match self.program_counter + 2 < RAM_SIZE as u16 {
            true => self.program_counter + 2,
            false => 0,
        };
        let instruction = self.decode(opcode);
        self.execute(instruction);

        //do this part 60 times a second
        self.display();
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
    JUMP {nnn: u16 },   //1nnn where nnn is a 12 bit value (lowest 12 bits of the instruction)
    ClearScreen, //clears the screen, does not take any arguments
    AddToRegister { x: u8, kk: u8 },
    LoadRegisterVx { x: u8,kk: u8 }, //6xkk puts the value kk into Vx
    SetIndexRegister { nnn: u16 },  //ANNN set index register I to nnn
    DISPLAY{ x: u8, y: u8, n: u8}, //DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
}

fn main() {
    let b = RomBuffer::new("./testlogo.ch8");
    let mut c = CPU::new(b);

    loop {
        c.cycle();
    }
}
