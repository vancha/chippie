use crate::constants::RAM_SIZE;

///The ram of the chip8 cpu, it uses big endian and has the following layout:
///0x000 start of chip-8 ram
///0x000 to 0x080 holds for fontset
///0x200 start of most chip-8 programs
///0x600 start of eti 660 chip8 programs
///0xfff end of chip8 ram
#[derive(Debug, Copy, Clone)]
pub struct Ram {
    pub bytes: [u8; RAM_SIZE as usize],
}
impl Ram {
    /// Returns the ram with the fontset already loaded
    pub fn with_fonts() -> Self {
        let mut ram = Self {
            bytes: [0; RAM_SIZE as usize],
        };
        // The fontset
        // This is basically a collection of bytes that make up numbers when written out in binary.
        // To understand them, write them out in binary so you can visually see what they represent.
        //the first row of bytes is 0xF0, 0x90, 0x90, 0x90, 0xF0. Written in binary that makes
        //
        // 1111
        // 1001
        // 1001
        // 1001
        // 1111
        //
        // You can see how the ones trace out a 0, the same way the second row of bytes will trace out a 1
        // and so on and so forth.

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
            0xF0, 0x80, 0xF0, 0x80, 0x80, //f
        ];

        for (idx, value) in ram.bytes[0..fontset.len()].iter_mut().enumerate() {
            *value = fontset[idx];
        }
        ram
    }

    ///returns a value from ram
    pub fn get_byte(self, index: u16) -> u8 {
        self.bytes[index as usize]
        //((self.bytes[index as usize] as u16) << 8) | self.bytes[(index + 1) as usize] as u16
    }

    pub fn get_opcode(self, index: u16) -> u16 {
          ((self.bytes[index as usize] as u16) << 8) | self.bytes[(index + 1) as usize] as u16
    }

    pub fn set(&mut self, index: u16, value: u8) {
        self.bytes[index as usize] = value;
    }
}
