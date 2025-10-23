/// A list of every instruction in the chip8 language
/// nnn is a hexadecimal memory address, it's 12 bits long
/// nn is a hexadecimal byte, it's 8 bits
/// n is what's called a "nibble", it's 4 bits
/// X and Y are registers
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
