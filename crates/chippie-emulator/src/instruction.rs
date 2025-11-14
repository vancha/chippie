/// # A list of every instruction in the chip8 language
/// ## nnn
/// a hexadecimal memory address, it's 12 bits long
/// ## nn
/// a hexadecimal byte, 8 bits
/// ## n
/// a "nibble" 4 bits
/// ## X and Y
/// Registers
#[derive(Debug, PartialEq)]
pub enum Instruction {
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

impl Instruction {
    /// Takes two bytes, and decodes what instruction they represent
    pub fn new(opcode: u16) -> Self {
        match Self::get_nibble(opcode, 0) {
            0x0 => match Self::last_byte(opcode) {
                0xE0 => Instruction::ClearScreen,
                0xEE => Instruction::ReturnFromSubroutine,
                _ => Instruction::Noop, //panic!("Unimplemented opcode: {:#04x}", opcode),
            },
            0x1 => Instruction::Jump {
                nnn: Self::oxxx(opcode),
            },
            0x2 => Instruction::CallSubroutineAtNNN {
                nnn: Self::oxxx(opcode),
            },
            0x3 => Instruction::SkipNextInstructionIfXIsKK {
                x: Self::get_nibble(opcode, 1),
                kk: Self::last_byte(opcode),
            },
            0x4 => Instruction::SkipNextInstructionIfXIsNotKK {
                x: Self::get_nibble(opcode, 1),
                kk: Self::last_byte(opcode),
            },
            0x5 => Instruction::SkipNextInstructionIfXIsY {
                x: Self::get_nibble(opcode, 1),
                y: Self::get_nibble(opcode, 2),
            },
            0x6 => Instruction::LoadRegisterX {
                x: Self::get_nibble(opcode, 1),
                kk: Self::last_byte(opcode),
            },
            0x7 => Instruction::AddToRegisterX {
                x: Self::get_nibble(opcode, 1),
                kk: Self::last_byte(opcode),
            },
            0x8 => match Self::get_nibble(opcode, 3) {
                0x0 => Instruction::LoadRegisterXIntoY {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },
                0x1 => Instruction::LoadXOrYinX {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },
                0x2 => Instruction::LoadXAndYInX {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },
                0x3 => Instruction::LoadXXorYInX {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },

                0x4 => Instruction::AddYToX {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },
                0x5 => Instruction::SubYFromX {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },
                0x6 => Instruction::ShiftXRight1 {
                    x: Self::get_nibble(opcode, 1),
                },
                0x7 => Instruction::SubXFromY {
                    x: Self::get_nibble(opcode, 1),
                    y: Self::get_nibble(opcode, 2),
                },

                0xE => Instruction::ShiftXLeft1 {
                    x: Self::get_nibble(opcode, 1),
                },
                _ => {
                    panic!("some other 8xxx thingy")
                }
            },
            0x9 => Instruction::SkipNextInstructionIfXIsNotY {
                x: Self::get_nibble(opcode, 1),
                y: Self::get_nibble(opcode, 2),
            },
            0xA => Instruction::SetIndexRegister {
                nnn: Self::oxxx(opcode),
            },
            0xB => Instruction::JumpToAddressPlusV0 {
                nnn: Self::oxxx(opcode),
            },
            0xC => Instruction::SetXToRandom {
                x: Self::get_nibble(opcode, 1),
                kk: Self::last_byte(opcode),
            },
            0xD => Instruction::Display {
                x: Self::get_nibble(opcode, 1),
                y: Self::get_nibble(opcode, 2),
                n: Self::get_nibble(opcode, 3),
            },
            0xE => match Self::last_byte(opcode) {
                0xA1 => Instruction::SkipIfVxNotPressed {
                    x: Self::get_nibble(opcode, 1),
                },
                0x9E => Instruction::SkipIfVxPressed {
                    x: Self::get_nibble(opcode, 1),
                },
                _ => {
                    panic!("unimplemented opcode: 0x{opcode:04x}");
                }
            },
            0xF => match Self::last_byte(opcode) {
                0x0A => Instruction::WaitForKeyPressed {
                    x: Self::get_nibble(opcode, 1),
                },
                0x07 => Instruction::SetXToDelayTimer {
                    x: Self::get_nibble(opcode, 1),
                },
                0x15 => Instruction::SetDelayTimerToX {
                    x: Self::get_nibble(opcode, 1),
                },
                0x18 => Instruction::SetSoundTimerToX {
                    x: Self::get_nibble(opcode, 1),
                },
                0x1E => Instruction::AddXtoI {
                    x: Self::get_nibble(opcode, 1),
                },
                0x29 => Instruction::SetIToSpriteX {
                    x: Self::get_nibble(opcode, 1),
                },
                0x33 => Instruction::LoadBCDOfX {
                    x: Self::get_nibble(opcode, 1),
                },
                0x55 => Instruction::Write0ThroughX {
                    x: Self::get_nibble(opcode, 1),
                },
                0x65 => Instruction::Load0ThroughX {
                    x: Self::get_nibble(opcode, 1),
                },
                _ => {
                    panic!("unimplemented opcode: 0x{opcode:06x}");
                }
            },
            _ => {
                panic!("cannot decode,opcode not implemented: {opcode:04x}")
            }
        }
    }

    /// A nibble is 4 bits, so this returns the first 4 bits of an opcode
    fn get_nibble(opcode: u16, nth: u8) -> u8 {
        assert!(nth < 4);
        ((opcode >> (12 - 4 * nth)) & 0xf) as u8
    }
    /// Returns the last full byte byte of an opcode
    fn last_byte(opcode: u16) -> u8 {
        (opcode & 0xff) as u8
    }
    /// Returns the the last 12 bits of an opcode
    fn oxxx(opcode: u16) -> u16 {
        opcode & 0xfff
    }
}
