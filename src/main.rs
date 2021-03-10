use byteorder::{BigEndian, ByteOrder, LittleEndian};
///0x000 start of chip-8 ram
///0x000 to 0x080 reserved for fontset
///0x200 start of most chip-8 programs
///0x600 start of eti 660 chip8 programs
///0xfff end of chip8 ram
#[derive(Debug, Copy, Clone)]
struct RAM {
    bytes: [u8; 4096],
}
impl RAM {
    fn read_rom(location: &str) {}

    fn new() -> Self {
        //let mut bytes = [0; 4096];
        //bytes[0x201] = 0xE0; //placeholder for clearscreen instruction. this should be filled with rom data
        Self { bytes: [0;4096] }
    }
    fn get(self, index: u16) -> u16 {
        return ((self.bytes[index as usize] as u16) << 8)
            | self.bytes[(index + 1) as usize] as u16;
    }
    fn read_bytes(&self,start:usize,end:usize)->&[u8] {
        &self.bytes[start..end]
    }
}

struct RomBuffer {
    data: Vec<u16>,
    buffer: Vec<u8>,
}
impl RomBuffer {
    fn new(file: &str) -> Self {
        let mut data: Vec<u16> = vec![];
        let buffer: Vec<u8> = std::fs::read(file).unwrap();
        for (y, x) in buffer.chunks(2).enumerate() {
            let number = ((x[0] as u16) << 8) | x[1] as u16; // this might be wrong, maybe I don't want to convert endianness here
            data.push(number);
        }
        for uux in &data {
            println!("{:#06x}", uux);
        }
        RomBuffer {
            data: data,
            buffer: buffer,
        }
    }
}
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

    vindex: u16, //the seperate 16-bit I register,generally only used to store memory addresses in the lowest (rightmost) 12 bits
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
            10 => self.va,
            11 => self.vb,
            12 => self.vc,
            13 => self.vd,
            14 => self.ve,
            15 => self.vf,
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
            10 => {
                self.va = value;
            }
            11 => {
                self.vb = value;
            }
            12 => {
                self.vc = value;
            }
            13 => {
                self.vd = value;
            }
            14 => {
                self.ve = value;
            }
            15 => {
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
struct CPU {
    display: [bool; 2048],
    program_counter: u16, //starts at 0x200, the start of the non-reserved memory
    memory: RAM,
    registers: Registers,
    stack: Stack, //stack for keeping track of where to return to after subroutine, can go into 16 nested subroutines before stackoverflow
    stackpointer: u8, //only contains indexes to locations in the stack, so 0 through 15
                  //framebuffer:[bool;64*32],//*x,y) addressable memory array which indicates if pixels are on or off
}

impl CPU {
    fn display(&self){
        for x in 0..64 {
            for y in 0..64 {

            }
        }
    }
    fn fetch(&self, ram: &RAM) -> u16 {
        ram.get(self.program_counter)
    }
    fn xooo(&self, code: u16) -> u8 {
        ((code >> 12) & 0xF) as u8
    }
    fn oxoo(&self,code: u16) -> u8 {
       ((code >> 8 ) & 0xf) as u8
    }
    fn ooxo(&self,code:u16)->u8 {
       ((code >> 4) & 0xf) as u8 
    }
    fn ooox(&self,code:u16)->u8 {
       (code as u8) & 0xf 
    }
    fn ooxx(&self, code: u16) -> u8 {
       (code & 0xff) as u8 
    }
    fn oxxx(&self, code: u16) -> u16 {
       code & 0xfff 
    }

    fn decode(&self, opcode: u16) -> Instruction {
        println!("{:#06x}", opcode);

        match self.xooo(opcode) {
            0x0 => {
                match self.ooxx(opcode) {
                    0xE0 => {
                        println!("sending clear screen instruction");
                        Instruction::CLEAR_SCREEN
                    }
                    _ => panic!("what's going on {:#06x}",opcode),
                }
            }
            0x1 => Instruction::JUMP(self.oxxx(opcode)),
            0x6 => Instruction::LOAD_REGISTER_VX(self.oxoo(opcode), self.ooxx(opcode)),
            0x7 => Instruction::ADD_TO_REGISTER(self.oxoo(opcode), self.ooxx(opcode)),
            0xA => Instruction::SET_INDEX_REGISTER(self.oxxx(opcode)),
            0xD => Instruction::DISPLAY(self.oxoo(opcode),self.ooxo(opcode),self.ooox(opcode)),
            _ => {panic!("cannot decode. ") } 
    }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::JUMP(x) => {
                self.program_counter = x;
                println!("jumping to location {}", x);
            }
            Instruction::ADD_TO_REGISTER(x, y) => {
                let tmp = self.registers.get_register(x) + y;
                self.registers.set_register(x, tmp);
                println!("adding {} to register {}", y, x);
            }
            Instruction::CLEAR_SCREEN => {
                self.display.iter_mut().for_each(|x| *x = false);
                println!("clearing the screen");
            }
            Instruction::LOAD_REGISTER_VX(x, y) => {
                println!("setting value {} to register {}",y,x);
                self.registers.set_register(x, y);
            }
            Instruction::SET_INDEX_REGISTER(x) => {
                println!("Setting index register to {}",x);
                self.registers.set_index_register(x);
            }
            Instruction::DISPLAY(vx, vy, n) => {
                let value1 = self.registers.get_register(vx);
                let value2 = self.registers.get_register(vy);
                let start = self.registers.get_index_register() as usize;
                let end = start + (n as usize);
                let values = self.memory.read_bytes(start,end);
                for x in values {
                    println!("byte read: {:#04x}",x);
                }
                //println!("displaying. got {} from register{}, {} from register {}, and setting {} to {}",value1,vx, value2, vy,n,n);
            }
            _ => {
                panic!("unimplemented instruction");
            }
        }
    }

    fn cycle(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let opcode = self.fetch(&self.memory);
        self.program_counter = match self.program_counter + 2 < 4096 {
            true => self.program_counter + 2,
            false => 0,
        };
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn new(rom: RomBuffer) -> Self {
        let mut memory = RAM::new();
        for (x, y) in rom.buffer.iter().enumerate() {
            //println!("{:#04x}",y);
            memory.bytes[0x200 + x] = *y;
            //add all these bytes into memory, starting at 200
        }

        Self {
            display: [false; 2048],
            program_counter: 0x200,
            registers: Registers::new(),
            memory: memory,
            stack: Stack::new(),
            stackpointer: 0,
        }
    }
}

///nnn is a hexadecimal memory address,nn is a hexadecimal byte, n refers to a nibble, and X and Y
///are registeres
enum Instruction {
    JUMP(u16),    //1nnn where nnn is a 12 bit value (lowest 12 bits of the instruction)
    CLEAR_SCREEN, //00E0
    LOAD_REGISTER_VX(u8, u8), //6xkk puts the value kk into Vx
    ADD_TO_REGISTER(u8, u8), //7xnn add value kk to vx, then store result in vx
    SET_INDEX_REGISTER(u16), //ANNN set index register I to nnn
    DISPLAY(u8, u8, u8), //DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
}

fn main() {
    let b = RomBuffer::new("/home/vancha/Documenten/rust/chip8_emulator/ibmlogo.ch8");
    let mut c = CPU::new(b);

    while true {
        c.cycle();
    }
}
