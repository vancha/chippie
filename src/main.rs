use byteorder::{ByteOrder, LittleEndian,BigEndian};
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
}
struct RomBuffer {
    data:Vec<u16>,
    buffer:Vec<u8>,
}
impl RomBuffer {
    fn new(file:&str)->Self {
        let mut data:Vec<u16> = vec![];
        let buffer = std::fs::read(file).unwrap();
        for (y,x) in buffer.chunks(2).enumerate() {
            let number = ((x[0] as u16) << 8) | x[1] as u16;// this might be wrong, maybe I don't want to convert endianness here
            data.push(number);  
        }
        for uux in &data {
            println!("{:#06x}",uux);
        }
        RomBuffer {data:data,buffer:buffer }
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

    vindex: u16, //the seperate 16-bit I register,generally only used to store memory addresses in the loswest (rightmost) 12 bits
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
}

impl RAM {
    fn new() -> Self {
        RAM { bytes: [0; 4096] }
    }
    fn get(self,index:u16)->u16 {
            return ((self.bytes[index as usize] as u16) << 8) | self.bytes[(index+1)as usize] as u16;
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
    display:[bool;2048],
    program_counter: u16, //starts at 0x200, the start of the non-reserved memory
    memory: RAM,
    registers: Registers,
    stack: Stack, //stack for keeping track of where to return to after subroutine, can go into 16 nested subroutines before stackoverflow
    stackpointer: u8, //only contains indexes to locations in the stack, so 0 through 15
                  //framebuffer:[bool;64*32],//*x,y) addressable memory array which indicates if pixels are on or off
}

impl CPU {
    fn fetch(&self, ram: &RAM) -> u16 {
        //ram[self.program_counter]
        ram.get(self.program_counter)
    }

    fn decode(&self, opcode: u16) -> Instruction {//this does not work yet, to be implemented
        //println!("")

        Instruction::CLEAR_SCREEN
    }

    fn execute(&mut self,instruction:Instruction) {
        match instruction {
            Instruction::JUMP(x) => {
                self.program_counter = x;
                println!("jumping to location {}",x);
            },
            Instruction::CLEAR_SCREEN => {
                self.display.iter_mut().for_each(|x| *x = false);
                println!("clearing the screen");
            },
            Instruction::LOAD_REGISTER_VX(x,y) => {
                
            }
            _ => {
                println!("unimplemented instruction");
            },
        }
    }

    fn cycle(&mut self) {
        let opcode = self.fetch(&self.memory);
        
        self.program_counter.wrapping_add(2);//incremented program counter by 2
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn new() -> Self {
        CPU {
            display:[false;2048],
            program_counter: 0x200,
            registers: Registers::new(),
            memory: RAM::new(),
            stack: Stack::new(),
            stackpointer: 0,
        }
    }
}

///nnn is a hexadecimal memory address,nn is a hexadecimal byte, n refers to a nibble, and X and Y
///are registeres
enum Instruction{
    JUMP(u16),//1nnn where nnn is a 12 bit value (lowest 12 bits of the instruction)
    CLEAR_SCREEN,//00E0
    LOAD_REGISTER_VX(u8,u8),//6xkk puts the value kk into Vx
    ADD_TO_REGISTER(u8,u8),//7xnn add value kk to vx, then store result in vx
    SET_INDEX_REGISTER(u16),//ANNN set index register I to nnn
    DISPLAY(u8,u8,u8),//DXYN draws a sprite at coordinate from vx and vy, of width 8 and height n
}

fn main() {
    //let b = RomBuffer::new("/home/vancha/Documenten/rust/chip8_emulator/ibmlogo.ch8");
    let mut c = CPU::new();
    while true {
        c.cycle();
    }
}
