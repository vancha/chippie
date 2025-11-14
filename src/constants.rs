/// The width of the display in pixels
pub const DISPLAY_WIDTH: u8 = 64;
/// The height of the display in pixels
pub const DISPLAY_HEIGHT: u8 = 32;
/// The size of ram in bytes
pub const RAM_SIZE: u16 = 4096;
/// How many cycles the cpu advances for every frame. This decides how fast the cpu will run
pub const CYCLES_PER_FRAME: usize = 5;
/// For the regular chip 8 roms
pub const ROM_START_ADDRESS: u16 = 0x200;
/// Amount of registers CHIP-8 has
pub const NUM_REGISTERS: u8 = 16;
