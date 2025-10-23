use crate::constants::RAM_SIZE;

///The ram of the chip8 cpu, uses big endian, and is laid out in the following way:
///0x000 start of chip-8 ram
///0x000 to 0x080 reserved for fontset
///0x200 start of most chip-8 programs
///0x600 start of eti 660 chip8 programs
///0xfff end of chip8 ram
#[derive(Debug, Copy, Clone)]
pub struct Ram {
    bytes: [u8; RAM_SIZE],
}
impl Ram {
    /// Returns the ram with the fontset already loaded
    pub fn with_fonts() -> Self {
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
    pub fn get(self, index: u16) -> u16 {
        ((self.bytes[index as usize] as u16) << 8) | self.bytes[(index + 1) as usize] as u16
    }

    pub fn set(&mut self, index: u16, value: u8) {
        //((self.bytes[index as usize] as u16) << 8) | self.bytes[(index + 1) as usize] as u16
        self.bytes[index as usize] = value;
    }
}
